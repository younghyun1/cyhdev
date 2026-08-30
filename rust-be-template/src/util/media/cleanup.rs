//! Durable object-cleanup ledger insertion shared by media transactions.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use tracing::error;
use uuid::Uuid;

use crate::{
    features::accounts::service::account_service::AccountService,
    util::media::{
        object_store::ObjectLocation,
        persistence::{CleanupFailure, bounded_cleanup_error},
    },
};

pub const REASON_SUPERSEDED_PROFILE_PICTURE: &str = "superseded_profile_picture";
pub const REASON_PROFILE_PICTURE_HISTORY_PRUNED: &str = "profile_picture_history_pruned";
pub const REASON_PROFILE_PICTURE_DELETED: &str = "profile_picture_deleted";
pub const REASON_DELETED_PHOTOGRAPH_IMAGE: &str = "deleted_photograph_image";
pub const REASON_DELETED_PHOTOGRAPH_THUMBNAIL: &str = "deleted_photograph_thumbnail";
pub const REASON_DELETED_WASM_THUMBNAIL: &str = "deleted_wasm_thumbnail";
pub const REASON_SUPERSEDED_WASM_THUMBNAIL: &str = "superseded_wasm_thumbnail";
pub const MEDIA_CLEANUP_DATABASE_CHUNK_SIZE: usize = 64;
pub const MEDIA_CLEANUP_ENQUEUE_LIMIT: usize = 256;
pub const MEDIA_CLEANUP_RETRY_SCAN_LIMIT: i64 = 256;
pub const MEDIA_CLEANUP_RETRY_ATTEMPT_LIMIT: usize = 32;

/// One object URL that must survive its source metadata being changed or deleted.
pub struct MediaCleanupRequest {
    pub original_url: String,
    pub reason: &'static str,
    pub source_id: Uuid,
}

/// Resolved cleanup row safe to submit to the configured object store.
#[derive(Debug, Clone)]
pub struct DurableMediaCleanup {
    pub cleanup_id: Uuid,
    pub location: ObjectLocation,
    pub attempt_count: i32,
}

/// Rows enqueued by one transaction; invalid legacy URLs remain unresolved in SQL.
pub struct EnqueuedMediaCleanup {
    pub resolved: Vec<DurableMediaCleanup>,
    pub unresolved_count: usize,
}

/// One optimistic failure update applied only to the observed retry generation.
#[derive(Debug, Clone)]
pub struct MediaCleanupFailureUpdate {
    pub cleanup_id: Uuid,
    pub expected_attempt_count: i32,
    pub error: String,
}

/// Bounded retry candidate loaded from the durable ledger.
pub struct MediaCleanupRetryCandidate {
    pub cleanup: DurableMediaCleanup,
    pub last_attempt_at: Option<DateTime<Utc>>,
}

/// Resolved ledger row inserted after an unregistered compensation failure.
pub struct MediaCleanupFailureRegistration {
    pub location: ObjectLocation,
    pub original_url: String,
    pub reason: &'static str,
    pub source_id: Uuid,
    pub attempted_at: DateTime<Utc>,
    pub error: String,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MediaCleanupEnqueueReport {
    pub submitted: usize,
    pub inserted: usize,
    pub already_registered: usize,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MediaCleanupRetryReport {
    pub attempted: usize,
    pub remote_deleted: usize,
    pub remote_failed: usize,
    pub settlement: CleanupSettlement,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CleanupSettlement {
    pub finalized: usize,
    pub failures_recorded: usize,
    pub ledger_errors: usize,
}

/// Best-effort ledger settlement after direct post-commit object deletion.
pub async fn settle_durable_cleanup(
    account_service: &AccountService,
    cleanup_objects: Vec<DurableMediaCleanup>,
    cleaned: &[ObjectLocation],
    failures: &[CleanupFailure],
) -> CleanupSettlement {
    let mut pending = cleanup_objects
        .into_iter()
        .map(|cleanup| (cleanup.location.clone(), cleanup))
        .collect::<HashMap<_, _>>();
    let mut completed_ids = Vec::with_capacity(cleaned.len());
    for location in cleaned {
        let Some(cleanup) = pending.remove(location) else {
            continue;
        };
        completed_ids.push(cleanup.cleanup_id);
    }
    let mut failure_updates = Vec::with_capacity(failures.len());
    for failure in failures {
        let Some(cleanup) = pending.remove(&failure.location) else {
            continue;
        };
        failure_updates.push(MediaCleanupFailureUpdate {
            cleanup_id: cleanup.cleanup_id,
            expected_attempt_count: cleanup.attempt_count,
            error: bounded_cleanup_error(&failure.error),
        });
    }

    let mut settlement = CleanupSettlement::default();
    if !completed_ids.is_empty() {
        match account_service
            .complete_media_object_cleanups(&completed_ids)
            .await
        {
            Ok(finalized) => settlement.finalized = finalized,
            Err(source) => {
                settlement.ledger_errors = settlement
                    .ledger_errors
                    .saturating_add(completed_ids.len());
                error!(
                    error = %source,
                    cleanup_count = completed_ids.len(),
                    "Failed to bulk-finalize completed media cleanup"
                );
            }
        }
    }
    if !failure_updates.is_empty() {
        match account_service
            .record_media_object_cleanup_failures(Utc::now(), &failure_updates)
            .await
        {
            Ok(recorded) => settlement.failures_recorded = recorded,
            Err(source) => {
                settlement.ledger_errors = settlement
                    .ledger_errors
                    .saturating_add(failure_updates.len());
                error!(
                    error = %source,
                    cleanup_count = failure_updates.len(),
                    "Failed to bulk-record media cleanup attempts"
                );
            }
        }
    }
    settlement
}

pub fn is_supported_media_cleanup_reason(reason: &str) -> bool {
    matches!(
        reason,
        REASON_SUPERSEDED_PROFILE_PICTURE
            | REASON_PROFILE_PICTURE_HISTORY_PRUNED
            | REASON_PROFILE_PICTURE_DELETED
            | REASON_DELETED_PHOTOGRAPH_IMAGE
            | REASON_DELETED_PHOTOGRAPH_THUMBNAIL
            | REASON_DELETED_WASM_THUMBNAIL
            | REASON_SUPERSEDED_WASM_THUMBNAIL
    )
}
