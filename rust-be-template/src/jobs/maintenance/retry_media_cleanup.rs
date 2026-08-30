//! Bounded retry worker for durable object-store cleanup rows.

use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use diesel::{BoolExpressionMethods, ExpressionMethods, NullableExpressionMethods, QueryDsl};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use futures_util::{StreamExt, stream};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    init::state::ServerState,
    schema::media_object_cleanup,
    util::media::{
        object_store::{MediaObjectStore, ObjectLocation, S3MediaObjectStore},
        persistence::bounded_cleanup_error,
    },
};

const CLEANUP_SCAN_LIMIT: usize = 256;
const CLEANUP_SCAN_PAGE_SIZE: usize = 64;
const CLEANUP_ATTEMPT_LIMIT: usize = 32;
const CLEANUP_DELETE_CONCURRENCY: usize = 4;
const CLEANUP_RETRY_BASE_SECONDS: i64 = 30;
const CLEANUP_RETRY_MAX_SECONDS: i64 = 3_600;

struct CleanupCandidate {
    cleanup_id: Uuid,
    location: ObjectLocation,
    attempt_count: i32,
    last_attempt_at: Option<DateTime<Utc>>,
}

/// Deletes due resolved objects, preserving every failed row for a later retry.
pub async fn retry_media_object_cleanup(state: Arc<ServerState>) {
    let now = Utc::now();
    let candidates = match load_due_candidates(&state, now).await {
        Ok(candidates) => candidates,
        Err(source) => {
            error!(error = %source, "Failed to load durable media cleanup work");
            return;
        }
    };
    if candidates.is_empty() {
        return;
    }

    let store = S3MediaObjectStore::from_config(&state.aws_profile_picture_config);
    let outcomes = stream::iter(candidates)
        .map(|candidate| {
            let store = &store;
            async move {
                let result = store.delete(candidate.location.clone()).await;
                (candidate, result)
            }
        })
        .buffer_unordered(CLEANUP_DELETE_CONCURRENCY)
        .collect::<Vec<_>>()
        .await;

    let account_service = state.account_service();
    let mut completed = 0_usize;
    let mut failed = 0_usize;
    for (candidate, result) in outcomes {
        match result {
            Ok(()) => match account_service
                .complete_media_object_cleanup(candidate.cleanup_id)
                .await
            {
                Ok(true) => completed += 1,
                Ok(false) => {}
                Err(source) => error!(
                    error = %source,
                    cleanup_id = %candidate.cleanup_id,
                    "Failed to finalize retried media cleanup"
                ),
            },
            Err(object_error) => {
                failed += 1;
                let error_message = bounded_cleanup_error(&object_error);
                match account_service
                    .record_media_object_cleanup_failure(
                        candidate.cleanup_id,
                        candidate.attempt_count,
                        Utc::now(),
                        &error_message,
                    )
                    .await
                {
                    Ok(true) => {}
                    Ok(false) => warn!(
                        cleanup_id = %candidate.cleanup_id,
                        "Media cleanup row changed during retry"
                    ),
                    Err(source) => error!(
                        error = %source,
                        cleanup_id = %candidate.cleanup_id,
                        "Failed to record retried media cleanup failure"
                    ),
                }
            }
        }
    }

    info!(attempted = completed + failed, completed, failed, "Retried durable media cleanup");
}

async fn load_due_candidates(
    state: &ServerState,
    now: DateTime<Utc>,
) -> anyhow::Result<Vec<CleanupCandidate>> {
    let mut connection = state.get_conn().await?;
    let mut due = Vec::with_capacity(CLEANUP_ATTEMPT_LIMIT);
    let mut scanned = 0_usize;
    scan_never_attempted(&mut connection, &mut due, &mut scanned).await?;
    if due.len() < CLEANUP_ATTEMPT_LIMIT && scanned < CLEANUP_SCAN_LIMIT {
        scan_attempted(&mut connection, now, &mut due, &mut scanned).await?;
    }
    Ok(due)
}

async fn scan_never_attempted(
    connection: &mut AsyncPgConnection,
    due: &mut Vec<CleanupCandidate>,
    scanned: &mut usize,
) -> diesel::result::QueryResult<()> {
    let mut cursor: Option<(DateTime<Utc>, Uuid)> = None;
    while due.len() < CLEANUP_ATTEMPT_LIMIT && *scanned < CLEANUP_SCAN_LIMIT {
        let limit = CLEANUP_SCAN_PAGE_SIZE.min(CLEANUP_SCAN_LIMIT - *scanned);
        let mut query = resolved_cleanup_query()
            .filter(media_object_cleanup::media_object_cleanup_last_attempt_at.is_null());
        if let Some((created_at, cleanup_id)) = cursor {
            query = query.filter(
                media_object_cleanup::media_object_cleanup_created_at
                    .gt(created_at)
                    .or(media_object_cleanup::media_object_cleanup_created_at
                        .eq(created_at)
                        .and(media_object_cleanup::media_object_cleanup_id.gt(cleanup_id))),
            );
        }
        let rows = query
            .order((
                media_object_cleanup::media_object_cleanup_created_at.asc(),
                media_object_cleanup::media_object_cleanup_id.asc(),
            ))
            .limit(limit as i64)
            .select((
                media_object_cleanup::media_object_cleanup_id,
                media_object_cleanup::media_object_cleanup_bucket.assume_not_null(),
                media_object_cleanup::media_object_cleanup_key.assume_not_null(),
                media_object_cleanup::media_object_cleanup_attempt_count,
                media_object_cleanup::media_object_cleanup_created_at,
            ))
            .load::<(Uuid, String, String, i32, DateTime<Utc>)>(connection)
            .await?;
        let page_len = rows.len();
        *scanned += page_len;
        for (cleanup_id, bucket, key, attempt_count, created_at) in rows {
            cursor = Some((created_at, cleanup_id));
            due.push(CleanupCandidate {
                cleanup_id,
                location: ObjectLocation::new(bucket, key),
                attempt_count,
                last_attempt_at: None,
            });
            if due.len() == CLEANUP_ATTEMPT_LIMIT {
                break;
            }
        }
        if page_len < limit {
            break;
        }
    }
    Ok(())
}

async fn scan_attempted(
    connection: &mut AsyncPgConnection,
    now: DateTime<Utc>,
    due: &mut Vec<CleanupCandidate>,
    scanned: &mut usize,
) -> diesel::result::QueryResult<()> {
    let mut cursor: Option<(DateTime<Utc>, DateTime<Utc>, Uuid)> = None;
    while due.len() < CLEANUP_ATTEMPT_LIMIT && *scanned < CLEANUP_SCAN_LIMIT {
        let limit = CLEANUP_SCAN_PAGE_SIZE.min(CLEANUP_SCAN_LIMIT - *scanned);
        let mut query = resolved_cleanup_query()
            .filter(media_object_cleanup::media_object_cleanup_last_attempt_at.is_not_null());
        if let Some((last_attempt_at, created_at, cleanup_id)) = cursor {
            let last_attempt =
                media_object_cleanup::media_object_cleanup_last_attempt_at.assume_not_null();
            query = query.filter(
                last_attempt.gt(last_attempt_at).or(last_attempt
                    .eq(last_attempt_at)
                    .and(media_object_cleanup::media_object_cleanup_created_at.gt(created_at).or(
                        media_object_cleanup::media_object_cleanup_created_at
                            .eq(created_at)
                            .and(media_object_cleanup::media_object_cleanup_id.gt(cleanup_id)),
                    ))),
            );
        }
        let rows = query
            .order((
                media_object_cleanup::media_object_cleanup_last_attempt_at.asc(),
                media_object_cleanup::media_object_cleanup_created_at.asc(),
                media_object_cleanup::media_object_cleanup_id.asc(),
            ))
            .limit(limit as i64)
            .select((
                media_object_cleanup::media_object_cleanup_id,
                media_object_cleanup::media_object_cleanup_bucket.assume_not_null(),
                media_object_cleanup::media_object_cleanup_key.assume_not_null(),
                media_object_cleanup::media_object_cleanup_attempt_count,
                media_object_cleanup::media_object_cleanup_created_at,
                media_object_cleanup::media_object_cleanup_last_attempt_at.assume_not_null(),
            ))
            .load::<(Uuid, String, String, i32, DateTime<Utc>, DateTime<Utc>)>(connection)
            .await?;
        let page_len = rows.len();
        *scanned += page_len;
        for (cleanup_id, bucket, key, attempt_count, created_at, last_attempt_at) in rows {
            cursor = Some((last_attempt_at, created_at, cleanup_id));
            let candidate = CleanupCandidate {
                cleanup_id,
                location: ObjectLocation::new(bucket, key),
                attempt_count,
                last_attempt_at: Some(last_attempt_at),
            };
            if retry_is_due(&candidate, now) {
                due.push(candidate);
            }
            if due.len() == CLEANUP_ATTEMPT_LIMIT {
                break;
            }
        }
        if page_len < limit {
            break;
        }
    }
    Ok(())
}

fn resolved_cleanup_query(
) -> media_object_cleanup::BoxedQuery<'static, diesel::pg::Pg> {
    media_object_cleanup::table
        .filter(media_object_cleanup::media_object_cleanup_bucket.is_not_null())
        .filter(media_object_cleanup::media_object_cleanup_key.is_not_null())
        .into_boxed()
}

fn retry_is_due(candidate: &CleanupCandidate, now: DateTime<Utc>) -> bool {
    let Some(last_attempt_at) = candidate.last_attempt_at else {
        return true;
    };
    match last_attempt_at.checked_add_signed(retry_backoff(candidate.attempt_count)) {
        Some(next_attempt_at) => next_attempt_at <= now,
        None => false,
    }
}

fn retry_backoff(attempt_count: i32) -> Duration {
    let exponent = attempt_count.saturating_sub(1).clamp(0, 7) as u32;
    let seconds = (CLEANUP_RETRY_BASE_SECONDS * 2_i64.pow(exponent))
        .min(CLEANUP_RETRY_MAX_SECONDS);
    Duration::seconds(seconds)
}

#[cfg(test)]
mod tests {
    use super::{CleanupCandidate, retry_backoff, retry_is_due};
    use crate::util::media::object_store::ObjectLocation;
    use chrono::{Duration, Utc};
    use uuid::Uuid;

    #[test]
    fn retry_backoff_is_exponential_and_capped() {
        assert_eq!(retry_backoff(1), Duration::seconds(30));
        assert_eq!(retry_backoff(4), Duration::minutes(4));
        assert_eq!(retry_backoff(i32::MAX), Duration::hours(1));
    }

    #[test]
    fn retry_due_respects_the_attempt_deadline() {
        let now = Utc::now();
        let candidate = CleanupCandidate {
            cleanup_id: Uuid::now_v7(),
            location: ObjectLocation::new("bucket", "key"),
            attempt_count: 2,
            last_attempt_at: Some(now - Duration::seconds(59)),
        };
        assert!(!retry_is_due(&candidate, now));
        assert!(retry_is_due(&candidate, now + Duration::seconds(1)));
    }
}
