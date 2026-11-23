//! Typed rows used by bulk media-cleanup inserts and optimistic retries.

use std::collections::HashMap;

use jiff::Timestamp;
use uuid::Uuid;

use crate::{schema::media_object_cleanup, util::media::cleanup::MediaCleanupFailureUpdate};

#[derive(diesel::Insertable)]
#[diesel(table_name = media_object_cleanup)]
pub(super) struct NewFailedMediaObjectCleanup<'a> {
    pub(super) media_object_cleanup_bucket: &'a str,
    pub(super) media_object_cleanup_key: &'a str,
    pub(super) media_object_cleanup_original_url: &'a str,
    pub(super) media_object_cleanup_reason: &'a str,
    pub(super) media_object_cleanup_source_id: Uuid,
    pub(super) media_object_cleanup_attempt_count: i32,
    #[diesel(serialize_as = jiff_diesel::Timestamp)]
    pub(super) media_object_cleanup_last_attempt_at: Timestamp,
    pub(super) media_object_cleanup_last_error: &'a str,
}

#[derive(diesel::Insertable)]
#[diesel(table_name = media_object_cleanup)]
pub(super) struct RetriedMediaObjectCleanup {
    media_object_cleanup_id: Uuid,
    media_object_cleanup_bucket: Option<String>,
    media_object_cleanup_key: Option<String>,
    media_object_cleanup_original_url: String,
    media_object_cleanup_reason: String,
    media_object_cleanup_source_id: Uuid,
    media_object_cleanup_attempt_count: i32,
    #[diesel(serialize_as = jiff_diesel::Timestamp)]
    media_object_cleanup_created_at: Timestamp,
    #[diesel(serialize_as = jiff_diesel::NullableTimestamp)]
    media_object_cleanup_last_attempt_at: Option<Timestamp>,
    media_object_cleanup_last_error: Option<String>,
}

pub(super) type StoredMediaObjectCleanup = (
    Uuid,
    Option<String>,
    Option<String>,
    String,
    String,
    Uuid,
    i32,
    jiff_diesel::Timestamp,
);

pub(super) fn retry_row(
    row: StoredMediaObjectCleanup,
    failures_by_id: &HashMap<Uuid, &MediaCleanupFailureUpdate>,
    attempted_at: Timestamp,
) -> Option<RetriedMediaObjectCleanup> {
    let (cleanup_id, bucket, key, original_url, reason, source_id, attempt_count, created_at) = row;
    let failure = failures_by_id.get(&cleanup_id)?;
    if attempt_count != failure.expected_attempt_count {
        return None;
    }
    Some(RetriedMediaObjectCleanup {
        media_object_cleanup_id: cleanup_id,
        media_object_cleanup_bucket: bucket,
        media_object_cleanup_key: key,
        media_object_cleanup_original_url: original_url,
        media_object_cleanup_reason: reason,
        media_object_cleanup_source_id: source_id,
        media_object_cleanup_attempt_count: attempt_count.saturating_add(1),
        media_object_cleanup_created_at: created_at.to_jiff(),
        media_object_cleanup_last_attempt_at: Some(attempted_at),
        media_object_cleanup_last_error: Some(failure.error.clone()),
    })
}
