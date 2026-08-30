//! Bounded persistence operations for the durable media-cleanup ledger.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use diesel::{ExpressionMethods, NullableExpressionMethods, QueryDsl};
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    features::accounts::{
        error::AccountError,
        repository::account_repository::AccountRepository,
    },
    schema::media_object_cleanup,
    util::media::{
        cleanup::{
            DurableMediaCleanup, MEDIA_CLEANUP_DATABASE_CHUNK_SIZE,
            MEDIA_CLEANUP_RETRY_ATTEMPT_LIMIT, MEDIA_CLEANUP_RETRY_SCAN_LIMIT,
            MediaCleanupFailureRegistration, MediaCleanupFailureUpdate,
            MediaCleanupRetryCandidate,
        },
        object_store::ObjectLocation,
    },
};

use super::media_cleanup_records::{
    NewFailedMediaObjectCleanup, StoredMediaObjectCleanup, retry_row,
};

impl AccountRepository {
    pub async fn media_cleanup_retry_candidates(
        &self,
    ) -> Result<Vec<MediaCleanupRetryCandidate>, AccountError> {
        let mut connection = self.connection().await?;
        let never_limit = i64::try_from(MEDIA_CLEANUP_RETRY_ATTEMPT_LIMIT)
            .map_err(|_| AccountError::ProfileCleanupCountOverflow)?;
        let never_attempted = resolved_cleanup_query()
            .filter(media_object_cleanup::media_object_cleanup_last_attempt_at.is_null())
            .order((
                media_object_cleanup::media_object_cleanup_created_at.asc(),
                media_object_cleanup::media_object_cleanup_id.asc(),
            ))
            .limit(never_limit)
            .select((
                media_object_cleanup::media_object_cleanup_id,
                media_object_cleanup::media_object_cleanup_bucket.assume_not_null(),
                media_object_cleanup::media_object_cleanup_key.assume_not_null(),
                media_object_cleanup::media_object_cleanup_attempt_count,
            ))
            .load::<(Uuid, String, String, i32)>(&mut connection)
            .await
            .map_err(AccountError::Query)?;
        let mut candidates = never_attempted
            .into_iter()
            .map(|(cleanup_id, bucket, key, attempt_count)| MediaCleanupRetryCandidate {
                cleanup: DurableMediaCleanup {
                    cleanup_id,
                    location: ObjectLocation::new(bucket, key),
                    attempt_count,
                },
                last_attempt_at: None,
            })
            .collect::<Vec<_>>();
        if candidates.len() >= MEDIA_CLEANUP_RETRY_ATTEMPT_LIMIT {
            return Ok(candidates);
        }
        let scanned = i64::try_from(candidates.len())
            .map_err(|_| AccountError::ProfileCleanupCountOverflow)?;
        let attempted_limit = MEDIA_CLEANUP_RETRY_SCAN_LIMIT.saturating_sub(scanned);
        let attempted = resolved_cleanup_query()
            .filter(media_object_cleanup::media_object_cleanup_last_attempt_at.is_not_null())
            .order((
                media_object_cleanup::media_object_cleanup_last_attempt_at
                    .assume_not_null()
                    .asc(),
                media_object_cleanup::media_object_cleanup_created_at.asc(),
                media_object_cleanup::media_object_cleanup_id.asc(),
            ))
            .limit(attempted_limit)
            .select((
                media_object_cleanup::media_object_cleanup_id,
                media_object_cleanup::media_object_cleanup_bucket.assume_not_null(),
                media_object_cleanup::media_object_cleanup_key.assume_not_null(),
                media_object_cleanup::media_object_cleanup_attempt_count,
                media_object_cleanup::media_object_cleanup_last_attempt_at.assume_not_null(),
            ))
            .load::<(Uuid, String, String, i32, DateTime<Utc>)>(&mut connection)
            .await
            .map_err(AccountError::Query)?;
        candidates.extend(attempted.into_iter().map(
            |(cleanup_id, bucket, key, attempt_count, last_attempt_at)| {
                MediaCleanupRetryCandidate {
                    cleanup: DurableMediaCleanup {
                        cleanup_id,
                        location: ObjectLocation::new(bucket, key),
                        attempt_count,
                    },
                    last_attempt_at: Some(last_attempt_at),
                }
            },
        ));
        Ok(candidates)
    }

    pub async fn complete_media_object_cleanups(
        &self,
        cleanup_ids: &[Uuid],
    ) -> Result<usize, AccountError> {
        let mut cleanup_ids = cleanup_ids.to_vec();
        cleanup_ids.sort_unstable();
        cleanup_ids.dedup();
        let mut connection = self.connection().await?;
        connection
            .transaction::<usize, diesel::result::Error, _>(async move |connection| {
                let mut deleted = 0_usize;
                for chunk in cleanup_ids.chunks(MEDIA_CLEANUP_DATABASE_CHUNK_SIZE) {
                    deleted = deleted.saturating_add(
                        diesel::delete(
                            media_object_cleanup::table.filter(
                                media_object_cleanup::media_object_cleanup_id.eq_any(chunk),
                            ),
                        )
                        .execute(&mut *connection)
                        .await?,
                    );
                }
                Ok(deleted)
            })
            .await
            .map_err(AccountError::Mutation)
    }

    pub async fn record_media_object_cleanup_failures(
        &self,
        attempted_at: DateTime<Utc>,
        failures: &[MediaCleanupFailureUpdate],
    ) -> Result<usize, AccountError> {
        let mut failures = failures.to_vec();
        failures.sort_unstable_by_key(|failure| {
            (failure.cleanup_id, failure.expected_attempt_count)
        });
        failures.dedup_by_key(|failure| failure.cleanup_id);
        let mut connection = self.connection().await?;
        connection
            .transaction::<usize, diesel::result::Error, _>(async move |connection| {
                let mut updated = 0_usize;
                for chunk in failures.chunks(MEDIA_CLEANUP_DATABASE_CHUNK_SIZE) {
                    let failures_by_id = chunk
                        .iter()
                        .map(|failure| (failure.cleanup_id, failure))
                        .collect::<HashMap<_, _>>();
                    let cleanup_ids = failures_by_id.keys().copied().collect::<Vec<_>>();
                    let stored = media_object_cleanup::table
                        .filter(media_object_cleanup::media_object_cleanup_id.eq_any(cleanup_ids))
                        .select((
                            media_object_cleanup::media_object_cleanup_id,
                            media_object_cleanup::media_object_cleanup_bucket,
                            media_object_cleanup::media_object_cleanup_key,
                            media_object_cleanup::media_object_cleanup_original_url,
                            media_object_cleanup::media_object_cleanup_reason,
                            media_object_cleanup::media_object_cleanup_source_id,
                            media_object_cleanup::media_object_cleanup_attempt_count,
                            media_object_cleanup::media_object_cleanup_created_at,
                        ))
                        .for_update()
                        .load::<StoredMediaObjectCleanup>(&mut *connection)
                        .await?;
                    let retries = stored
                        .into_iter()
                        .filter_map(|row| retry_row(row, &failures_by_id, attempted_at))
                        .collect::<Vec<_>>();
                    if retries.is_empty() {
                        continue;
                    }
                    updated = updated.saturating_add(
                        diesel::insert_into(media_object_cleanup::table)
                            .values(&retries)
                            .on_conflict(media_object_cleanup::media_object_cleanup_id)
                            .do_update()
                            .set((
                                media_object_cleanup::media_object_cleanup_attempt_count.eq(
                                    diesel::upsert::excluded(
                                        media_object_cleanup::media_object_cleanup_attempt_count,
                                    ),
                                ),
                                media_object_cleanup::media_object_cleanup_last_attempt_at.eq(
                                    diesel::upsert::excluded(
                                        media_object_cleanup::media_object_cleanup_last_attempt_at,
                                    ),
                                ),
                                media_object_cleanup::media_object_cleanup_last_error.eq(
                                    diesel::upsert::excluded(
                                        media_object_cleanup::media_object_cleanup_last_error,
                                    ),
                                ),
                            ))
                            .execute(&mut *connection)
                            .await?,
                    );
                }
                Ok(updated)
            })
            .await
            .map_err(AccountError::Mutation)
    }

    pub async fn enqueue_media_cleanup_failures(
        &self,
        failures: &[MediaCleanupFailureRegistration],
    ) -> Result<usize, AccountError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<usize, diesel::result::Error, _>(async move |connection| {
                let mut inserted = 0_usize;
                for chunk in failures.chunks(MEDIA_CLEANUP_DATABASE_CHUNK_SIZE) {
                    let rows = chunk
                        .iter()
                        .map(|failure| NewFailedMediaObjectCleanup {
                            media_object_cleanup_bucket: failure.location.bucket(),
                            media_object_cleanup_key: failure.location.key(),
                            media_object_cleanup_original_url: &failure.original_url,
                            media_object_cleanup_reason: failure.reason,
                            media_object_cleanup_source_id: failure.source_id,
                            media_object_cleanup_attempt_count: 1,
                            media_object_cleanup_last_attempt_at: failure.attempted_at,
                            media_object_cleanup_last_error: &failure.error,
                        })
                        .collect::<Vec<_>>();
                    inserted = inserted.saturating_add(
                        diesel::insert_into(media_object_cleanup::table)
                            .values(rows)
                            .on_conflict_do_nothing()
                            .execute(&mut *connection)
                            .await?,
                    );
                }
                Ok(inserted)
            })
            .await
            .map_err(AccountError::Mutation)
    }
}

fn resolved_cleanup_query() -> media_object_cleanup::BoxedQuery<'static, diesel::pg::Pg> {
    media_object_cleanup::table
        .filter(media_object_cleanup::media_object_cleanup_bucket.is_not_null())
        .filter(media_object_cleanup::media_object_cleanup_key.is_not_null())
        .into_boxed()
}
