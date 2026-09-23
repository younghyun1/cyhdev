use chrono::{DateTime, Utc};
use serde_derive::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::features::blog::domain::{comment::CommentResponse, comment_page::CommentCursor};

/// Query values that request the page after the last comment received.
#[derive(Serialize, ToSchema)]
pub struct CommentCursorResponse {
    pub after_created_at: DateTime<Utc>,
    pub after_comment_id: Uuid,
}

impl From<CommentCursor> for CommentCursorResponse {
    fn from(value: CommentCursor) -> Self {
        Self {
            after_created_at: value.created_at,
            after_comment_id: value.comment_id,
        }
    }
}

/// One oldest-first page of a post's comments. Deleted comments appear as
/// tombstones with empty content so their replies stay attached.
#[derive(Serialize, ToSchema)]
pub struct CommentPageResponse {
    pub comments: Vec<CommentResponse>,
    #[schema(required)]
    pub next_cursor: Option<CommentCursorResponse>,
}
