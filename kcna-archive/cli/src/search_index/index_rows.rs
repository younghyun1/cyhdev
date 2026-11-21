//! Database reads behind the snapshot writer: documents by keyset page or by
//! identifier, the copies folded into them, and the identifiers used to find
//! stale index entries.

use std::collections::{HashMap, HashSet};

use diesel::{ExpressionMethods, NullableExpressionMethods, QueryDsl, dsl::count_star, pg::Pg};
use diesel_async::RunQueryDsl;
use kcna_archive_core::search::request::CopyRef;
use time::{Date, OffsetDateTime};
use uuid::Uuid;

use crate::{
    database::{
        pool::PgPool,
        schema::{kcna_categories, kcna_documents},
        types::{Language, PublicationPrecision, Source},
    },
    error::{Error, Result},
};

/// Rows fetched per query: bounds memory while keeping round trips rare.
pub(super) const PAGE_ROWS: i64 = 2_000;

/// A stored row as the snapshot writer reads it.
pub(super) struct IndexRow {
    pub(super) id: Uuid,
    pub(super) title: String,
    pub(super) body: Option<String>,
    pub(super) date: Date,
    pub(super) precision: PublicationPrecision,
    pub(super) language: Language,
    pub(super) source: Source,
    pub(super) canonical_url: String,
    pub(super) category: Option<String>,
    pub(super) source_record_id: Option<String>,
    pub(super) metadata: serde_json::Value,
    pub(super) translation_group: Option<Uuid>,
    /// Set when the row is a copy of another document, which is then the one
    /// indexed.
    pub(super) canonical: Option<Uuid>,
    pub(super) updated_at: OffsetDateTime,
}

/// Which rows a read returns.
pub(super) enum Rows<'a> {
    /// One keyset page in identifier order, optionally only rows changed
    /// after the watermark.
    Page {
        after: Option<Uuid>,
        changed_after: Option<OffsetDateTime>,
    },
    /// The rows with these identifiers, at most [`PAGE_ROWS`] of them.
    Ids(&'a [Uuid]),
}

/// A copy of a preferred document, with the translation group it carries.
pub(super) struct CopyRow {
    pub(super) copy: CopyRef,
    pub(super) translation_group: Option<Uuid>,
}

type RowTuple = (
    Uuid,
    String,
    Option<String>,
    Date,
    PublicationPrecision,
    Language,
    Source,
    String,
    Option<String>,
    Option<String>,
    serde_json::Value,
    Option<Uuid>,
    Option<Uuid>,
    OffsetDateTime,
);

async fn connection(
    pool: &PgPool,
) -> Result<
    diesel_async::pooled_connection::bb8::PooledConnection<'_, diesel_async::AsyncPgConnection>,
> {
    pool.get()
        .await
        .map_err(|error| Error::DatabasePool(error.to_string()))
}

/// Reads stored rows in identifier order.
pub(super) async fn load(pool: &PgPool, rows: Rows<'_>) -> Result<Vec<IndexRow>> {
    let mut connection = connection(pool).await?;
    let mut statement = kcna_documents::table
        .left_join(kcna_categories::table)
        .select((
            kcna_documents::document_id,
            kcna_documents::document_title,
            kcna_documents::document_body,
            kcna_documents::document_publication_date,
            kcna_documents::document_publication_precision,
            kcna_documents::document_language,
            kcna_documents::document_source,
            kcna_documents::document_canonical_url,
            kcna_categories::category_name.nullable(),
            kcna_documents::document_source_record_id,
            kcna_documents::document_source_metadata,
            kcna_documents::document_translation_group_id,
            kcna_documents::document_canonical_document_id,
            kcna_documents::document_updated_at,
        ))
        .order(kcna_documents::document_id.asc())
        .limit(PAGE_ROWS)
        .into_boxed::<Pg>();
    match rows {
        Rows::Page {
            after,
            changed_after,
        } => {
            if let Some(after) = after {
                statement = statement.filter(kcna_documents::document_id.gt(after));
            }
            if let Some(changed_after) = changed_after {
                statement = statement.filter(kcna_documents::document_updated_at.gt(changed_after));
            }
        }
        Rows::Ids(ids) => {
            statement = statement.filter(kcna_documents::document_id.eq_any(ids));
        }
    }
    let loaded = statement.load::<RowTuple>(&mut connection).await?;
    Ok(loaded
        .into_iter()
        .map(
            |(
                id,
                title,
                body,
                date,
                precision,
                language,
                source,
                canonical_url,
                category,
                source_record_id,
                metadata,
                translation_group,
                canonical,
                updated_at,
            )| IndexRow {
                id,
                title,
                body,
                date,
                precision,
                language,
                source,
                canonical_url,
                category,
                source_record_id,
                metadata,
                translation_group,
                canonical,
                updated_at,
            },
        )
        .collect())
}

/// The copies pointing at each preferred document, in a stable order.
pub(super) async fn copies(
    pool: &PgPool,
    preferred: &[Uuid],
) -> Result<HashMap<Uuid, Vec<CopyRow>>> {
    if preferred.is_empty() {
        return Ok(HashMap::new());
    }
    let mut connection = connection(pool).await?;
    let loaded = kcna_documents::table
        .filter(kcna_documents::document_canonical_document_id.eq_any(preferred))
        .select((
            kcna_documents::document_canonical_document_id,
            kcna_documents::document_id,
            kcna_documents::document_source,
            kcna_documents::document_canonical_url,
            kcna_documents::document_translation_group_id,
        ))
        .order(kcna_documents::document_id.asc())
        .load::<(Option<Uuid>, Uuid, Source, String, Option<Uuid>)>(&mut connection)
        .await?;
    let mut copies: HashMap<Uuid, Vec<CopyRow>> = HashMap::new();
    for (canonical, id, source, url, translation_group) in loaded {
        if let Some(canonical) = canonical {
            copies.entry(canonical).or_default().push(CopyRow {
                copy: CopyRef { id, source, url },
                translation_group,
            });
        }
    }
    for rows in copies.values_mut() {
        rows.sort_by_key(|row| (row.copy.source.code(), row.copy.id));
    }
    Ok(copies)
}

/// Stored documents that have an index entry of their own.
pub(super) async fn count(pool: &PgPool) -> Result<u64> {
    let mut connection = connection(pool).await?;
    let stored = kcna_documents::table
        .filter(kcna_documents::document_canonical_document_id.is_null())
        .select(count_star())
        .get_result::<i64>(&mut connection)
        .await?;
    Ok(u64::try_from(stored).unwrap_or(0))
}

/// Every stored identifier: 16 bytes each, a few megabytes for the corpus.
pub(super) async fn identifiers(pool: &PgPool) -> Result<HashSet<Uuid>> {
    let mut connection = connection(pool).await?;
    let mut known = HashSet::new();
    let mut after: Option<Uuid> = None;
    loop {
        let mut statement = kcna_documents::table
            .filter(kcna_documents::document_canonical_document_id.is_null())
            .select(kcna_documents::document_id)
            .order(kcna_documents::document_id.asc())
            .limit(PAGE_ROWS * 25)
            .into_boxed::<Pg>();
        if let Some(after) = after {
            statement = statement.filter(kcna_documents::document_id.gt(after));
        }
        let ids = statement.load::<Uuid>(&mut connection).await?;
        let Some(last) = ids.last() else { break };
        after = Some(*last);
        known.extend(ids);
    }
    Ok(known)
}
