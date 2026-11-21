//! Stores the raw responses behind a batch of documents and links each
//! document to the response it was read from.

use std::collections::{HashMap, HashSet};

use diesel::{ExpressionMethods, SelectableHelper, pg::Pg, upsert::excluded};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use crate::database::{
    domain::DocumentBundle,
    models::{NewDocumentRetrieval, NewRetrieval},
    schema::{kcna_document_retrievals, kcna_retrievals},
    store::{NaturalKey, natural_key},
};

type RetrievalKey = (String, Vec<u8>);
type QueryResult<T> = std::result::Result<T, diesel::result::Error>;

#[derive(Clone, Debug, diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = kcna_retrievals, check_for_backend(Pg))]
struct StoredRetrieval {
    retrieval_id: Uuid,
    retrieval_requested_url: String,
    retrieval_raw_sha256: Vec<u8>,
}

/// Stores each distinct raw body once and links it to its documents.
pub async fn write(
    connection: &mut AsyncPgConnection,
    changed: &[(&DocumentBundle, [u8; 32])],
    ids: &HashMap<NaturalKey, Uuid>,
) -> QueryResult<()> {
    let mut unique = HashSet::with_capacity(changed.len());
    let retrievals = changed
        .iter()
        .filter(|(bundle, _)| {
            unique.insert((bundle.requested_url.as_ref().to_owned(), bundle.raw_sha256))
        })
        .map(|(bundle, _)| NewRetrieval::from_bundle(bundle))
        .collect::<Vec<_>>();
    let stored = diesel::insert_into(kcna_retrievals::table)
        .values(&retrievals)
        .on_conflict((
            kcna_retrievals::retrieval_requested_url,
            kcna_retrievals::retrieval_raw_sha256,
        ))
        .do_update()
        .set((
            kcna_retrievals::retrieval_final_url.eq(excluded(kcna_retrievals::retrieval_final_url)),
            kcna_retrievals::retrieval_http_status
                .eq(excluded(kcna_retrievals::retrieval_http_status)),
            kcna_retrievals::retrieval_media_type
                .eq(excluded(kcna_retrievals::retrieval_media_type)),
            kcna_retrievals::retrieval_charset.eq(excluded(kcna_retrievals::retrieval_charset)),
            kcna_retrievals::retrieval_response_headers
                .eq(excluded(kcna_retrievals::retrieval_response_headers)),
            kcna_retrievals::retrieval_manifest_metadata
                .eq(excluded(kcna_retrievals::retrieval_manifest_metadata)),
        ))
        .returning(StoredRetrieval::as_returning())
        .get_results::<StoredRetrieval>(connection)
        .await?;
    let retrieval_ids = stored
        .into_iter()
        .map(|row| {
            (
                (row.retrieval_requested_url, row.retrieval_raw_sha256),
                row.retrieval_id,
            )
        })
        .collect::<HashMap<RetrievalKey, Uuid>>();
    let links = changed
        .iter()
        .filter_map(|(bundle, _)| {
            let retrieval_key = (
                bundle.requested_url.as_ref().to_owned(),
                bundle.raw_sha256.to_vec(),
            );
            ids.get(&natural_key(bundle))
                .zip(retrieval_ids.get(&retrieval_key))
                .map(|(document_id, retrieval_id)| NewDocumentRetrieval {
                    document_retrieval_document_id: *document_id,
                    document_retrieval_retrieval_id: *retrieval_id,
                })
        })
        .collect::<Vec<_>>();
    diesel::insert_into(kcna_document_retrievals::table)
        .values(&links)
        .on_conflict((
            kcna_document_retrievals::document_retrieval_document_id,
            kcna_document_retrievals::document_retrieval_retrieval_id,
        ))
        .do_nothing()
        .execute(connection)
        .await?;
    Ok(())
}
