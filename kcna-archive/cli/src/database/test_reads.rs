//! Reads of stored documents for the PostgreSQL round-trip checks.
//!
//! The website reads the snapshot, not the database, so nothing outside the
//! tests asks PostgreSQL for a single document. The checks still need to see
//! what the importer stored and how its rows are linked.

use diesel::{
    BoolExpressionMethods, ExpressionMethods, NullableExpressionMethods, OptionalExtension,
    QueryDsl,
};
use diesel_async::RunQueryDsl;
use time::Date;
use uuid::Uuid;

use crate::{
    database::{
        pool::PgPool,
        schema::{kcna_categories, kcna_documents},
        types::{Language, Source},
    },
    error::{Error, Result},
};

#[derive(Clone)]
pub struct DocumentStore {
    pool: PgPool,
}

/// The stored facts the checks compare.
#[derive(Clone, Debug)]
pub struct StoredDocument {
    pub id: Uuid,
    pub title: String,
    pub body: Option<String>,
    pub publication_date: Date,
    pub language: Language,
    pub category: Option<String>,
    pub translation_group_id: Option<Uuid>,
    /// The preferred copy when this document is a duplicate from another source.
    pub canonical_document_id: Option<Uuid>,
}

/// The same article in another language.
#[derive(Clone, Debug)]
pub struct Translation {
    pub id: Uuid,
    pub language: Language,
}

/// The same article as published by another source.
#[derive(Clone, Debug)]
pub struct Copy {
    pub id: Uuid,
    pub source: Source,
}

type DocumentRow = (
    Uuid,
    String,
    Option<String>,
    Date,
    Language,
    Option<String>,
    Option<Uuid>,
    Option<Uuid>,
);

impl DocumentStore {
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn connection(
        &self,
    ) -> Result<
        diesel_async::pooled_connection::bb8::PooledConnection<'_, diesel_async::AsyncPgConnection>,
    > {
        self.pool
            .get()
            .await
            .map_err(|error| Error::DatabasePool(error.to_string()))
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<StoredDocument>> {
        let mut connection = self.connection().await?;
        let row = kcna_documents::table
            .left_join(kcna_categories::table)
            .filter(kcna_documents::document_id.eq(id))
            .select((
                kcna_documents::document_id,
                kcna_documents::document_title,
                kcna_documents::document_body,
                kcna_documents::document_publication_date,
                kcna_documents::document_language,
                kcna_categories::category_name.nullable(),
                kcna_documents::document_translation_group_id,
                kcna_documents::document_canonical_document_id,
            ))
            .first::<DocumentRow>(&mut connection)
            .await
            .optional()?;
        Ok(row.map(
            |(
                id,
                title,
                body,
                publication_date,
                language,
                category,
                translation_group_id,
                canonical_document_id,
            )| StoredDocument {
                id,
                title,
                body,
                publication_date,
                language,
                category,
                translation_group_id,
                canonical_document_id,
            },
        ))
    }

    /// The other language editions of a document's translation group, one per
    /// language, preferring the canonical copy.
    pub async fn translations(&self, document: &StoredDocument) -> Result<Vec<Translation>> {
        let Some(group) = document.translation_group_id else {
            return Ok(Vec::new());
        };
        let mut connection = self.connection().await?;
        let rows = kcna_documents::table
            .filter(kcna_documents::document_translation_group_id.eq(group))
            .filter(kcna_documents::document_language.ne(document.language))
            .select((
                kcna_documents::document_id,
                kcna_documents::document_language,
                kcna_documents::document_canonical_document_id,
            ))
            .limit(64)
            .load::<(Uuid, Language, Option<Uuid>)>(&mut connection)
            .await?;
        let mut translations: Vec<(Translation, bool)> = Vec::new();
        for (id, language, canonical) in rows {
            let preferred = canonical.is_none();
            match translations
                .iter_mut()
                .find(|(existing, _)| existing.language == language)
            {
                Some(existing) if preferred && !existing.1 => {
                    *existing = (Translation { id, language }, preferred);
                }
                Some(_) => {}
                None => translations.push((Translation { id, language }, preferred)),
            }
        }
        let mut translations = translations
            .into_iter()
            .map(|(translation, _)| translation)
            .collect::<Vec<_>>();
        translations.sort_unstable_by_key(|translation| translation.language.code());
        Ok(translations)
    }

    /// The other copies of the same article: the canonical one and everything
    /// that points at it, except the document itself.
    pub async fn copies(&self, document: &StoredDocument) -> Result<Vec<Copy>> {
        let root = document.canonical_document_id.unwrap_or(document.id);
        let mut connection = self.connection().await?;
        let rows = kcna_documents::table
            .filter(
                kcna_documents::document_id
                    .eq(root)
                    .or(kcna_documents::document_canonical_document_id.eq(root)),
            )
            .filter(kcna_documents::document_id.ne(document.id))
            .select((kcna_documents::document_id, kcna_documents::document_source))
            .limit(16)
            .load::<(Uuid, Source)>(&mut connection)
            .await?;
        let mut copies = rows
            .into_iter()
            .map(|(id, source)| Copy { id, source })
            .collect::<Vec<_>>();
        copies.sort_unstable_by_key(|copy| copy.source.code());
        Ok(copies)
    }
}
