//! Corpus-wide counts recorded in the snapshot manifest.

use diesel::{
    ExpressionMethods, QueryDsl,
    dsl::{count_star, max},
};
use diesel_async::RunQueryDsl;
use time::OffsetDateTime;

use crate::{
    database::{
        pool::PgPool,
        schema::{kcna_documents, kcna_retrievals, kcna_translation_groups},
    },
    error::{Error, Result},
};

/// Counts describing the stored corpus.
#[derive(Clone, Copy, Debug)]
pub(super) struct CorpusStats {
    pub(super) documents: i64,
    /// Documents folded into another source's copy of the same article.
    pub(super) copies: i64,
    pub(super) translation_groups: i64,
    pub(super) last_retrieved_at: Option<OffsetDateTime>,
}

pub(super) async fn read(pool: &PgPool) -> Result<CorpusStats> {
    let mut connection = match pool.get().await {
        Ok(connection) => connection,
        Err(error) => return Err(Error::DatabasePool(error.to_string())),
    };
    let documents = kcna_documents::table
        .select(count_star())
        .get_result::<i64>(&mut connection)
        .await?;
    let copies = kcna_documents::table
        .filter(kcna_documents::document_canonical_document_id.is_not_null())
        .select(count_star())
        .get_result::<i64>(&mut connection)
        .await?;
    let translation_groups = kcna_translation_groups::table
        .select(count_star())
        .get_result::<i64>(&mut connection)
        .await?;
    let last_retrieved_at = kcna_retrievals::table
        .select(max(kcna_retrievals::retrieval_retrieved_at))
        .get_result::<Option<OffsetDateTime>>(&mut connection)
        .await?;
    Ok(CorpusStats {
        documents,
        copies,
        translation_groups,
        last_retrieved_at,
    })
}
