//! Transport-independent forum input validation.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::features::forum::{
    domain::{models::{ForumReplyCursor, ForumTimestampCursor, ForumTopicCursor}, validation::{ForumPageSize, ForumSearch}},
    error::ForumError,
};

pub(super) fn page_size(limit: Option<u16>, default: u16) -> Result<ForumPageSize, ForumError> {
    ForumPageSize::try_new(limit.unwrap_or(default)).map_err(|_| ForumError::InvalidPageSize)
}

pub(super) fn search(value: Option<String>) -> Result<Option<ForumSearch>, ForumError> {
    match value {
        Some(value) if value.trim().is_empty() => Ok(None),
        Some(value) => ForumSearch::try_new(value).map(Some).map_err(|_| ForumError::InvalidSearch),
        None => Ok(None),
    }
}

pub(super) fn topic_cursor(
    pinned: Option<bool>, activity: Option<DateTime<Utc>>, topic_id: Option<Uuid>,
) -> Result<Option<ForumTopicCursor>, ForumError> {
    match (pinned, activity, topic_id) {
        (Some(is_pinned), Some(last_activity_at), Some(topic_id)) => Ok(Some(ForumTopicCursor { is_pinned, last_activity_at, topic_id })),
        (None, None, None) => Ok(None),
        _ => Err(ForumError::InvalidCursor),
    }
}

pub(super) fn reply_cursor(created_at: Option<DateTime<Utc>>, reply_id: Option<Uuid>) -> Result<Option<ForumReplyCursor>, ForumError> {
    match (created_at, reply_id) {
        (Some(created_at), Some(reply_id)) => Ok(Some(ForumReplyCursor { created_at, reply_id })),
        (None, None) => Ok(None),
        _ => Err(ForumError::InvalidCursor),
    }
}

pub(super) fn timestamp_cursor(created_at: Option<DateTime<Utc>>, item_id: Option<Uuid>) -> Result<Option<ForumTimestampCursor>, ForumError> {
    match (created_at, item_id) {
        (Some(created_at), Some(item_id)) => Ok(Some(ForumTimestampCursor { created_at, item_id })),
        (None, None) => Ok(None),
        _ => Err(ForumError::InvalidCursor),
    }
}

pub(super) fn revision(value: i32) -> Result<i32, ForumError> {
    (value >= 1).then_some(value).ok_or(ForumError::InvalidRevision)
}
