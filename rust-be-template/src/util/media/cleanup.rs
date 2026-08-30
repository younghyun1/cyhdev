//! Durable object-cleanup ledger insertion shared by media transactions.

use std::collections::HashMap;

use chrono::Utc;
use diesel::{ExpressionMethods, NullableExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use tracing::error;
use uuid::Uuid;

use crate::{
    features::accounts::service::account_service::AccountService,
    schema::media_object_cleanup,
    util::{
        media::{
            object_store::ObjectLocation,
            persistence::{CleanupFailure, bounded_cleanup_error},
        },
        s3::AWS_S3_BUCKET_NAME,
    },
};

pub const REASON_SUPERSEDED_PROFILE_PICTURE: &str = "superseded_profile_picture";
pub const REASON_PROFILE_PICTURE_HISTORY_PRUNED: &str = "profile_picture_history_pruned";
pub const REASON_PROFILE_PICTURE_DELETED: &str = "profile_picture_deleted";
pub const REASON_DELETED_PHOTOGRAPH_IMAGE: &str = "deleted_photograph_image";
pub const REASON_DELETED_PHOTOGRAPH_THUMBNAIL: &str = "deleted_photograph_thumbnail";
pub const REASON_DELETED_WASM_THUMBNAIL: &str = "deleted_wasm_thumbnail";
pub const REASON_SUPERSEDED_WASM_THUMBNAIL: &str = "superseded_wasm_thumbnail";

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
    let mut settlement = CleanupSettlement::default();
    let mut pending = cleanup_objects
        .into_iter()
        .map(|cleanup| (cleanup.location.clone(), cleanup))
        .collect::<HashMap<_, _>>();

    for location in cleaned {
        let Some(cleanup) = pending.remove(location) else {
            continue;
        };
        match account_service
            .complete_media_object_cleanup(cleanup.cleanup_id)
            .await
        {
            Ok(true) => settlement.finalized += 1,
            Ok(false) => {}
            Err(source) => {
                settlement.ledger_errors += 1;
                error!(
                error = %source,
                cleanup_id = %cleanup.cleanup_id,
                "Failed to finalize completed media cleanup"
                );
            }
        }
    }

    for failure in failures {
        let Some(cleanup) = pending.remove(&failure.location) else {
            continue;
        };
        let error_message = bounded_cleanup_error(&failure.error);
        match account_service
            .record_media_object_cleanup_failure(
                cleanup.cleanup_id,
                cleanup.attempt_count,
                Utc::now(),
                &error_message,
            )
            .await
        {
            Ok(true) => settlement.failures_recorded += 1,
            Ok(false) => {}
            Err(source) => {
                settlement.ledger_errors += 1;
                error!(
                error = %source,
                cleanup_id = %cleanup.cleanup_id,
                "Failed to record media cleanup attempt"
                );
            }
        }
    }
    settlement
}

#[derive(diesel::Insertable)]
#[diesel(table_name = media_object_cleanup)]
struct NewMediaObjectCleanup {
    media_object_cleanup_bucket: Option<String>,
    media_object_cleanup_key: Option<String>,
    media_object_cleanup_original_url: String,
    media_object_cleanup_reason: &'static str,
    media_object_cleanup_source_id: Uuid,
}

/// Enqueues cleanup before the caller clears or deletes authoritative metadata.
pub async fn enqueue_media_cleanup(
    connection: &mut diesel_async::AsyncPgConnection,
    requests: Vec<MediaCleanupRequest>,
) -> diesel::result::QueryResult<EnqueuedMediaCleanup> {
    if requests.is_empty() {
        return Ok(EnqueuedMediaCleanup {
            resolved: Vec::new(),
            unresolved_count: 0,
        });
    }

    let mut rows = Vec::with_capacity(requests.len());
    let mut resolved_keys = Vec::with_capacity(requests.len());
    let mut unresolved_count = 0_usize;
    for request in requests {
        let location = ObjectLocation::from_public_s3_url(
            AWS_S3_BUCKET_NAME,
            &request.original_url,
        );
        let (bucket, key) = match location {
            Some(location) => {
                resolved_keys.push(location.key().to_owned());
                (
                    Some(location.bucket().to_owned()),
                    Some(location.key().to_owned()),
                )
            }
            None => {
                unresolved_count += 1;
                (None, None)
            }
        };
        rows.push(NewMediaObjectCleanup {
            media_object_cleanup_bucket: bucket,
            media_object_cleanup_key: key,
            media_object_cleanup_original_url: request.original_url,
            media_object_cleanup_reason: request.reason,
            media_object_cleanup_source_id: request.source_id,
        });
    }

    diesel::insert_into(media_object_cleanup::table)
        .values(&rows)
        .on_conflict_do_nothing()
        .execute(&mut *connection)
        .await?;

    if resolved_keys.is_empty() {
        return Ok(EnqueuedMediaCleanup {
            resolved: Vec::new(),
            unresolved_count,
        });
    }

    resolved_keys.sort_unstable();
    resolved_keys.dedup();
    let resolved = media_object_cleanup::table
        .filter(media_object_cleanup::media_object_cleanup_bucket.eq(AWS_S3_BUCKET_NAME))
        .filter(media_object_cleanup::media_object_cleanup_key.eq_any(resolved_keys))
        .filter(media_object_cleanup::media_object_cleanup_bucket.is_not_null())
        .filter(media_object_cleanup::media_object_cleanup_key.is_not_null())
        .select((
            media_object_cleanup::media_object_cleanup_id,
            media_object_cleanup::media_object_cleanup_bucket.assume_not_null(),
            media_object_cleanup::media_object_cleanup_key.assume_not_null(),
            media_object_cleanup::media_object_cleanup_attempt_count,
        ))
        .load::<(Uuid, String, String, i32)>(&mut *connection)
        .await?
        .into_iter()
        .map(|(cleanup_id, bucket, key, attempt_count)| DurableMediaCleanup {
            cleanup_id,
            location: ObjectLocation::new(bucket, key),
            attempt_count,
        })
        .collect();

    Ok(EnqueuedMediaCleanup {
        resolved,
        unresolved_count,
    })
}
