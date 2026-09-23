//! Batched persistence for the buffered blog view counter.

use diesel::{
    ExpressionMethods, OptionalExtension, QueryDsl,
    sql_types::{Array, BigInt, Uuid as SqlUuid},
};
use diesel_async::RunQueryDsl;
use tracing::warn;
use uuid::Uuid;

use crate::schema::posts;

use super::super::error::BlogError;
use super::blog_repository::BlogRepository;

/// Rows per statement; two array binds keep each statement's size fixed.
const VIEW_DELTA_CHUNK_SIZE: usize = 256;

/// Diesel has no `UPDATE ... FROM unnest(...)`; both arrays are bound values.
const APPLY_VIEW_DELTAS_SQL: &str = "\
UPDATE posts AS post \
SET post_view_count = post.post_view_count + view_delta.delta \
FROM unnest($1::uuid[], $2::bigint[]) AS view_delta(post_id, delta) \
WHERE post.post_id = view_delta.post_id";

impl BlogRepository {
    /// Applies buffered deltas chunk by chunk and returns the chunks that
    /// committed. A failed chunk is logged and stays buffered for the next
    /// flush; a delta for a deleted post applies to no row and is dropped.
    pub async fn apply_view_deltas(
        &self,
        pending: &[(Uuid, i64)],
    ) -> Result<Vec<(Uuid, i64)>, BlogError> {
        let mut connection = self.connection().await?;
        let mut applied = Vec::with_capacity(pending.len());
        for chunk in pending.chunks(VIEW_DELTA_CHUNK_SIZE) {
            let post_ids = chunk
                .iter()
                .map(|(post_id, _)| *post_id)
                .collect::<Vec<_>>();
            let deltas = chunk.iter().map(|(_, delta)| *delta).collect::<Vec<_>>();
            let result = diesel::sql_query(APPLY_VIEW_DELTAS_SQL)
                .bind::<Array<SqlUuid>, _>(post_ids)
                .bind::<Array<BigInt>, _>(deltas)
                .execute(&mut connection)
                .await;
            match result {
                Ok(_) => applied.extend_from_slice(chunk),
                Err(error) => {
                    warn!(chunk_size = chunk.len(), %error, "Blog view-delta chunk remains queued")
                }
            }
        }
        Ok(applied)
    }

    /// Persists one view directly when the buffer cannot admit the post.
    pub async fn increment_view(&self, post_id: Uuid) -> Result<(), BlogError> {
        let mut connection = self.connection().await?;
        diesel::update(posts::table.find(post_id))
            .set(posts::post_view_count.eq(posts::post_view_count + 1_i64))
            .execute(&mut connection)
            .await?;
        Ok(())
    }

    pub async fn post_view_count(&self, post_id: Uuid) -> Result<Option<i64>, BlogError> {
        let mut connection = self.connection().await?;
        posts::table
            .find(post_id)
            .select(posts::post_view_count)
            .first::<i64>(&mut connection)
            .await
            .optional()
            .map_err(BlogError::Database)
    }
}
