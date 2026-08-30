//! Authorized durable-media reconciliation use cases.

use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use futures_util::{StreamExt, stream};
use uuid::Uuid;

use crate::features::accounts::{
    domain::lifecycle::{ResolvedMediaCleanup, UnresolvedMediaCleanup},
    error::AccountError,
    service::account_service::AccountService,
};
use crate::util::{
    media::{
        cleanup::{
            MEDIA_CLEANUP_ENQUEUE_LIMIT, MEDIA_CLEANUP_RETRY_ATTEMPT_LIMIT,
            MediaCleanupEnqueueReport, MediaCleanupFailureRegistration, MediaCleanupFailureUpdate,
            MediaCleanupRetryReport, is_supported_media_cleanup_reason, settle_durable_cleanup,
        },
        persistence::{CleanupFailure, MAX_CLEANUP_ERROR_CHARS, bounded_cleanup_error},
    },
    s3::AWS_S3_BUCKET_NAME,
};

const CLEANUP_DELETE_CONCURRENCY: usize = 4;
const CLEANUP_RETRY_BASE_SECONDS: i64 = 30;
const CLEANUP_RETRY_MAX_SECONDS: i64 = 3_600;

impl AccountService {
    pub async fn complete_media_object_cleanups(
        &self,
        cleanup_ids: &[Uuid],
    ) -> Result<usize, AccountError> {
        self.repository
            .complete_media_object_cleanups(cleanup_ids)
            .await
    }

    pub async fn record_media_object_cleanup_failures(
        &self,
        attempted_at: DateTime<Utc>,
        failures: &[MediaCleanupFailureUpdate],
    ) -> Result<usize, AccountError> {
        let failures = failures
            .iter()
            .map(|failure| MediaCleanupFailureUpdate {
                cleanup_id: failure.cleanup_id,
                expected_attempt_count: failure.expected_attempt_count,
                error: bounded_failure_message(&failure.error),
            })
            .collect::<Vec<_>>();
        self.repository
            .record_media_object_cleanup_failures(attempted_at, &failures)
            .await
    }

    /// Durably records object deletions which failed before a ledger row existed.
    pub async fn enqueue_media_cleanup_failures(
        &self,
        source_id: Uuid,
        reason: &'static str,
        failures: &[CleanupFailure],
    ) -> Result<MediaCleanupEnqueueReport, AccountError> {
        if failures.len() > MEDIA_CLEANUP_ENQUEUE_LIMIT {
            return Err(AccountError::MediaCleanupBatchTooLarge {
                max_items: MEDIA_CLEANUP_ENQUEUE_LIMIT,
            });
        }
        if failures.is_empty() {
            return Ok(MediaCleanupEnqueueReport::default());
        }
        if !is_supported_media_cleanup_reason(reason) {
            return Err(AccountError::InvalidMediaCleanupReason);
        }
        let attempted_at = Utc::now();
        let mut registrations = Vec::with_capacity(failures.len());
        for failure in failures {
            let bucket = failure.location.bucket();
            let key = failure.location.key();
            let original_url = format!("s3://{bucket}/{key}");
            if bucket.is_empty()
                || bucket.len() > 255
                || key.is_empty()
                || key.len() > 1_024
                || original_url.len() > 4_096
            {
                return Err(AccountError::InvalidMediaCleanupLocation);
            }
            registrations.push(MediaCleanupFailureRegistration {
                location: failure.location.clone(),
                original_url,
                reason,
                source_id,
                attempted_at,
                error: bounded_cleanup_error(&failure.error),
            });
        }
        let inserted = self
            .repository
            .enqueue_media_cleanup_failures(&registrations)
            .await?;
        Ok(MediaCleanupEnqueueReport {
            submitted: registrations.len(),
            inserted,
            already_registered: registrations.len().saturating_sub(inserted),
        })
    }

    /// Executes one bounded retry pass through the injected object-store port.
    pub async fn retry_media_object_cleanup(
        &self,
        now: DateTime<Utc>,
    ) -> Result<MediaCleanupRetryReport, AccountError> {
        let candidates = self
            .repository
            .media_cleanup_retry_candidates()
            .await?
            .into_iter()
            .filter(|candidate| {
                retry_is_due(
                    candidate.cleanup.attempt_count,
                    candidate.last_attempt_at,
                    now,
                )
            })
            .take(MEDIA_CLEANUP_RETRY_ATTEMPT_LIMIT)
            .map(|candidate| candidate.cleanup)
            .collect::<Vec<_>>();
        if candidates.is_empty() {
            return Ok(MediaCleanupRetryReport::default());
        }
        let cleanup_objects = candidates.clone();
        let store = Arc::clone(&self.media_object_store);
        let outcomes = stream::iter(candidates)
            .map(|candidate| {
                let store = Arc::clone(&store);
                async move {
                    let result = store.delete(candidate.location.clone()).await;
                    (candidate.location, result)
                }
            })
            .buffer_unordered(CLEANUP_DELETE_CONCURRENCY)
            .collect::<Vec<_>>()
            .await;
        let mut cleaned = Vec::new();
        let mut failures = Vec::new();
        for (location, result) in outcomes {
            match result {
                Ok(()) => cleaned.push(location),
                Err(error) => failures.push(CleanupFailure { location, error }),
            }
        }
        let settlement = settle_durable_cleanup(self, cleanup_objects, &cleaned, &failures).await;
        Ok(MediaCleanupRetryReport {
            attempted: cleaned.len().saturating_add(failures.len()),
            remote_deleted: cleaned.len(),
            remote_failed: failures.len(),
            settlement,
        })
    }

    pub async fn unresolved_media_cleanup(
        &self,
        requester_id: Uuid,
    ) -> Result<Vec<UnresolvedMediaCleanup>, AccountError> {
        let session_consistency = self.session_consistency.read().await;
        let records = self.repository.unresolved_media_cleanup(requester_id).await;
        drop(session_consistency);
        records
    }

    pub async fn resolve_media_cleanup(
        &self,
        requester_id: Uuid,
        cleanup_id: Uuid,
        expected_original_url: &str,
        bucket: &str,
        key: &str,
    ) -> Result<ResolvedMediaCleanup, AccountError> {
        validate_location(expected_original_url, bucket, key)?;
        let session_consistency = self.session_consistency.read().await;
        let resolution = self
            .repository
            .resolve_media_cleanup(requester_id, cleanup_id, expected_original_url, bucket, key)
            .await;
        drop(session_consistency);
        resolution
    }
}

fn bounded_failure_message(error: &str) -> String {
    let bounded = error
        .chars()
        .take(MAX_CLEANUP_ERROR_CHARS)
        .collect::<String>();
    if bounded.is_empty() {
        "object-store cleanup failed".to_owned()
    } else {
        bounded
    }
}

fn retry_is_due(
    attempt_count: i32,
    last_attempt_at: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
) -> bool {
    let Some(last_attempt_at) = last_attempt_at else {
        return true;
    };
    match last_attempt_at.checked_add_signed(retry_backoff(attempt_count)) {
        Some(next_attempt_at) => next_attempt_at <= now,
        None => false,
    }
}

fn retry_backoff(attempt_count: i32) -> Duration {
    let exponent = attempt_count.saturating_sub(1).clamp(0, 7);
    let exponent: u32 = u32::try_from(exponent).unwrap_or_default();
    let seconds = (CLEANUP_RETRY_BASE_SECONDS * 2_i64.pow(exponent)).min(CLEANUP_RETRY_MAX_SECONDS);
    Duration::seconds(seconds)
}

fn validate_location(original_url: &str, bucket: &str, key: &str) -> Result<(), AccountError> {
    let valid = !original_url.is_empty()
        && original_url.len() <= 4_096
        && bucket == AWS_S3_BUCKET_NAME
        && !key.is_empty()
        && key.len() <= 1_024;
    if valid {
        Ok(())
    } else {
        Err(AccountError::InvalidMediaCleanupLocation)
    }
}

#[cfg(test)]
mod tests {
    use super::{bounded_failure_message, retry_backoff, retry_is_due, validate_location};
    use crate::{features::accounts::error::AccountError, util::s3::AWS_S3_BUCKET_NAME};
    use chrono::{Duration, Utc};

    #[test]
    fn retry_backoff_is_exponential_and_capped() {
        assert_eq!(retry_backoff(1), Duration::seconds(30));
        assert_eq!(retry_backoff(4), Duration::minutes(4));
        assert_eq!(retry_backoff(i32::MAX), Duration::hours(1));
    }

    #[test]
    fn retry_due_uses_the_persisted_attempt_generation() {
        let now = Utc::now();
        assert!(!retry_is_due(2, Some(now - Duration::seconds(59)), now));
        assert!(retry_is_due(2, Some(now - Duration::seconds(60)), now));
        assert!(retry_is_due(0, None, now));
    }

    #[test]
    fn failure_messages_are_nonempty_and_character_bounded() {
        assert_eq!(bounded_failure_message(""), "object-store cleanup failed");
        assert_eq!(
            bounded_failure_message(&"🙂".repeat(3_000)).chars().count(),
            2_048
        );
    }

    #[test]
    fn reconciliation_accepts_only_the_configured_media_bucket() {
        let original_url = format!("s3://{AWS_S3_BUCKET_NAME}/images/example.avif");
        assert!(
            validate_location(&original_url, AWS_S3_BUCKET_NAME, "images/example.avif").is_ok()
        );
        assert!(matches!(
            validate_location(&original_url, "attacker-controlled", "images/example.avif"),
            Err(AccountError::InvalidMediaCleanupLocation)
        ));
    }
}
