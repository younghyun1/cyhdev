//! Links copies of one article published by several sources.
//!
//! The live site, the historical mirror, and the public dataset overlap, so a
//! search would list the same article up to three times. After an import this
//! pass points every duplicate at one preferred copy; search shows that copy
//! and names the other sources.

use std::collections::HashMap;

use diesel::{ExpressionMethods, QueryDsl, pg::Pg};
use diesel_async::{AsyncConnection, RunQueryDsl};
use time::Date;
use tracing::info;
use uuid::Uuid;

use crate::{
    database::{
        pool::PgPool,
        schema::kcna_documents,
        types::{Language, Source},
    },
    error::{Error, Result},
    importer::copy_plan::{ArticleKey, Member, Plan, plan, title_key},
};

const PAGE_ROWS: i64 = 20_000;
/// Identifiers per UPDATE statement.
const UPDATE_CHUNK: usize = 5_000;

diesel::define_sql_function! {
    fn transaction_timestamp() -> diesel::sql_types::Timestamptz;
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CopyReport {
    /// Articles held by more than one source.
    pub articles: usize,
    /// Documents whose link changed in this run.
    pub relinked: usize,
}

/// Recomputes every copy link and writes the ones that changed.
pub async fn refresh(pool: &PgPool) -> Result<CopyReport> {
    let mut connection = pool
        .get()
        .await
        .map_err(|error| Error::DatabasePool(error.to_string()))?;
    let mut groups: HashMap<ArticleKey, Vec<Member>> = HashMap::new();
    let mut after: Option<Uuid> = None;
    loop {
        let mut statement = kcna_documents::table
            .select((
                kcna_documents::document_id,
                kcna_documents::document_language,
                kcna_documents::document_publication_date,
                kcna_documents::document_source,
                kcna_documents::document_title,
                kcna_documents::document_body.is_not_null(),
                kcna_documents::document_content_sha256,
                kcna_documents::document_canonical_document_id,
            ))
            .order(kcna_documents::document_id.asc())
            .limit(PAGE_ROWS)
            .into_boxed::<Pg>();
        if let Some(after) = after {
            statement = statement.filter(kcna_documents::document_id.gt(after));
        }
        let rows = statement
            .load::<(
                Uuid,
                Language,
                Date,
                Source,
                String,
                bool,
                Vec<u8>,
                Option<Uuid>,
            )>(&mut connection)
            .await?;
        let Some(last) = rows.last() else { break };
        after = Some(last.0);
        for (id, language, date, source, title, has_body, content, canonical) in rows {
            let member = Member {
                id,
                source,
                content: has_body.then_some(content),
                canonical,
            };
            match title_key(&title) {
                Some(title) => groups
                    .entry((language, date, title))
                    .or_default()
                    .push(member),
                // An unusable title cannot identify an article; such a
                // document stands alone, under a key no other can share.
                None => groups
                    .entry((language, date, id.as_u128() as u64))
                    .or_default()
                    .push(member),
            }
        }
    }
    let Plan {
        articles,
        changes,
        touched,
    } = plan(groups.into_values());
    let relinked = changes.len();
    if relinked == 0 {
        return Ok(CopyReport { articles, relinked });
    }
    // Grouped by target so each statement sets one value for many rows.
    let mut by_target: HashMap<Option<Uuid>, Vec<Uuid>> = HashMap::new();
    for (id, target) in changes {
        by_target.entry(target).or_default().push(id);
    }
    connection
        .transaction::<(), diesel::result::Error, _>(async move |connection| {
            for (target, ids) in by_target {
                for chunk in ids.chunks(UPDATE_CHUNK) {
                    diesel::update(
                        kcna_documents::table.filter(kcna_documents::document_id.eq_any(chunk)),
                    )
                    .set((
                        kcna_documents::document_canonical_document_id.eq(target),
                        kcna_documents::document_updated_at.eq(transaction_timestamp()),
                    ))
                    .execute(connection)
                    .await?;
                }
            }
            // The preferred copy's index entry lists its sources, so it is
            // marked changed whenever its set of copies changes.
            for chunk in touched.chunks(UPDATE_CHUNK) {
                diesel::update(
                    kcna_documents::table.filter(kcna_documents::document_id.eq_any(chunk)),
                )
                .set(kcna_documents::document_updated_at.eq(transaction_timestamp()))
                .execute(connection)
                .await?;
            }
            Ok(())
        })
        .await?;
    info!(articles, relinked, "Linked copies across sources");
    Ok(CopyReport { articles, relinked })
}
