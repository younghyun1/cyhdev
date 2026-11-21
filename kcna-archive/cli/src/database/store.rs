//! Idempotent batch persistence for normalized documents and raw snapshots.

use std::collections::{HashMap, HashSet};

use diesel::{ExpressionMethods, QueryDsl, SelectableHelper, pg::Pg, upsert::excluded};
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use time::{Date, OffsetDateTime};
use uuid::Uuid;

use crate::{
    database::{
        domain::DocumentBundle,
        models::NewDocument,
        pool::PgPool,
        schema::kcna_documents,
        store_links, store_retrievals,
        types::{Language, PublicationPrecision, Source},
    },
    error::{Error, Result},
};

pub type NaturalKey = (Source, String, i16, Language);
type QueryResult<T> = std::result::Result<T, diesel::result::Error>;

/// Outcome of storing one batch.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Upserted {
    /// Documents of the batch that are stored afterwards.
    pub stored: usize,
    /// Of those, the ones that were new or differed from the stored row.
    pub changed: usize,
}

/// What is already stored for a document of the batch.
#[derive(Clone, Debug, diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = kcna_documents, check_for_backend(Pg))]
struct StoredState {
    document_id: Uuid,
    document_source: Source,
    document_canonical_url: String,
    document_element_index: i16,
    document_language: Language,
    document_publication_date: Date,
    document_publication_precision: PublicationPrecision,
    document_published_at: OffsetDateTime,
    document_import_fingerprint: Vec<u8>,
}

#[derive(Clone, Debug, diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = kcna_documents, check_for_backend(Pg))]
struct StoredDocument {
    document_id: Uuid,
    document_source: Source,
    document_canonical_url: String,
    document_element_index: i16,
    document_language: Language,
}

diesel::define_sql_function! {
    fn least(left: diesel::sql_types::Timestamptz, right: diesel::sql_types::Timestamptz) -> diesel::sql_types::Timestamptz;
}

diesel::define_sql_function! {
    fn greatest(left: diesel::sql_types::Timestamptz, right: diesel::sql_types::Timestamptz) -> diesel::sql_types::Timestamptz;
}

diesel::define_sql_function! {
    fn transaction_timestamp() -> diesel::sql_types::Timestamptz;
}

/// Stores one bounded batch in a single transaction.
///
/// Documents whose fingerprint equals the stored one are left untouched, so a
/// routine re-import writes only what is new or changed.
pub async fn upsert_batch(pool: &PgPool, bundles: &[DocumentBundle]) -> Result<Upserted> {
    if bundles.is_empty() {
        return Ok(Upserted::default());
    }
    let mut connection = pool
        .get()
        .await
        .map_err(|error| Error::DatabasePool(error.to_string()))?;
    let changed = connection
        .transaction::<usize, diesel::result::Error, _>(async move |connection| {
            let stored = stored_states(connection, bundles).await?;
            let changed = bundles
                .iter()
                .map(|bundle| (bundle, bundle.fingerprint()))
                .filter(|(bundle, fingerprint)| {
                    stored
                        .get(&natural_key(bundle))
                        .is_none_or(|state| state.document_import_fingerprint != fingerprint)
                })
                .collect::<Vec<_>>();
            if changed.is_empty() {
                return Ok(0);
            }
            delete_redated(connection, &stored, &changed).await?;
            let ids = write_documents(connection, &changed).await?;
            store_retrievals::write(connection, &changed, &ids).await?;
            Ok(changed.len())
        })
        .await?;
    Ok(Upserted {
        stored: bundles.len(),
        changed,
    })
}

/// Loads the stored rows for the batch, keyed by natural key.
async fn stored_states(
    connection: &mut AsyncPgConnection,
    bundles: &[DocumentBundle],
) -> QueryResult<HashMap<NaturalKey, StoredState>> {
    let urls = bundles
        .iter()
        .map(|bundle| bundle.canonical_url.as_ref())
        .collect::<HashSet<&str>>()
        .into_iter()
        .collect::<Vec<_>>();
    let rows = kcna_documents::table
        .filter(kcna_documents::document_canonical_url.eq_any(urls))
        .select(StoredState::as_select())
        .load::<StoredState>(connection)
        .await?;
    Ok(rows
        .into_iter()
        .map(|row| {
            (
                (
                    row.document_source,
                    row.document_canonical_url.clone(),
                    row.document_element_index,
                    row.document_language,
                ),
                row,
            )
        })
        .collect())
}

/// Removes stored documents whose publication time differs from the incoming one.
///
/// A document's UUIDv7 encodes its publication time and a check constraint
/// keeps the two within 25 ms, so a corrected date cannot be applied as an
/// update: the upsert would fail the constraint and roll back the whole batch.
/// Deleting first lets the insert mint an identifier for the corrected time.
async fn delete_redated(
    connection: &mut AsyncPgConnection,
    stored: &HashMap<NaturalKey, StoredState>,
    changed: &[(&DocumentBundle, [u8; 32])],
) -> QueryResult<()> {
    let redated = changed
        .iter()
        .filter_map(|(bundle, _)| {
            stored
                .get(&natural_key(bundle))
                .filter(|state| publication_changed(state, bundle))
                .map(|state| state.document_id)
        })
        .collect::<Vec<_>>();
    if redated.is_empty() {
        return Ok(());
    }
    diesel::delete(kcna_documents::table.filter(kcna_documents::document_id.eq_any(redated)))
        .execute(connection)
        .await?;
    Ok(())
}

/// Upserts the changed documents and returns their identifiers.
async fn write_documents(
    connection: &mut AsyncPgConnection,
    changed: &[(&DocumentBundle, [u8; 32])],
) -> QueryResult<HashMap<NaturalKey, Uuid>> {
    let groups = store_links::translation_groups(connection, changed).await?;
    let categories = store_links::categories(connection, changed).await?;
    let documents = changed
        .iter()
        .map(|(bundle, fingerprint)| {
            NewDocument::from_bundle(
                bundle,
                *fingerprint,
                bundle
                    .translation_group_key
                    .as_ref()
                    .and_then(|key| groups.get(key).copied()),
                store_links::category_key(bundle).and_then(|key| categories.get(&key).copied()),
            )
        })
        .collect::<Vec<_>>();
    let stored = diesel::insert_into(kcna_documents::table)
        .values(&documents)
        .on_conflict((
            kcna_documents::document_source,
            kcna_documents::document_canonical_url,
            kcna_documents::document_element_index,
            kcna_documents::document_language,
        ))
        .do_update()
        .set((
            kcna_documents::document_source_record_id
                .eq(excluded(kcna_documents::document_source_record_id)),
            kcna_documents::document_translation_group_id
                .eq(excluded(kcna_documents::document_translation_group_id)),
            kcna_documents::document_title.eq(excluded(kcna_documents::document_title)),
            kcna_documents::document_body.eq(excluded(kcna_documents::document_body)),
            kcna_documents::document_published_at
                .eq(excluded(kcna_documents::document_published_at)),
            kcna_documents::document_publication_precision
                .eq(excluded(kcna_documents::document_publication_precision)),
            kcna_documents::document_publication_date
                .eq(excluded(kcna_documents::document_publication_date)),
            kcna_documents::document_published_at_raw
                .eq(excluded(kcna_documents::document_published_at_raw)),
            kcna_documents::document_category_id.eq(excluded(kcna_documents::document_category_id)),
            kcna_documents::document_content_sha256
                .eq(excluded(kcna_documents::document_content_sha256)),
            kcna_documents::document_copyright_status
                .eq(excluded(kcna_documents::document_copyright_status)),
            kcna_documents::document_source_metadata
                .eq(excluded(kcna_documents::document_source_metadata)),
            kcna_documents::document_first_retrieved_at.eq(least(
                kcna_documents::document_first_retrieved_at,
                excluded(kcna_documents::document_first_retrieved_at),
            )),
            kcna_documents::document_last_retrieved_at.eq(greatest(
                kcna_documents::document_last_retrieved_at,
                excluded(kcna_documents::document_last_retrieved_at),
            )),
            kcna_documents::document_import_fingerprint
                .eq(excluded(kcna_documents::document_import_fingerprint)),
            kcna_documents::document_updated_at.eq(transaction_timestamp()),
        ))
        .returning(StoredDocument::as_returning())
        .get_results::<StoredDocument>(connection)
        .await?;
    Ok(stored
        .into_iter()
        .map(|row| {
            (
                (
                    row.document_source,
                    row.document_canonical_url,
                    row.document_element_index,
                    row.document_language,
                ),
                row.document_id,
            )
        })
        .collect())
}

fn publication_changed(stored: &StoredState, bundle: &DocumentBundle) -> bool {
    if stored.document_publication_precision != bundle.publication_precision
        || stored.document_publication_date != bundle.publication_date
    {
        return true;
    }
    // Day precisions are anchored to noon by the database, so only records
    // with a real time of day can differ within the same date.
    match bundle.publication_precision {
        PublicationPrecision::Day | PublicationPrecision::InferredDay => false,
        PublicationPrecision::Second | PublicationPrecision::Minute => {
            stored.document_published_at != bundle.published_at
        }
    }
}

pub fn natural_key(bundle: &DocumentBundle) -> NaturalKey {
    (
        bundle.source,
        bundle.canonical_url.as_ref().to_owned(),
        *bundle.element_index,
        bundle.language,
    )
}
