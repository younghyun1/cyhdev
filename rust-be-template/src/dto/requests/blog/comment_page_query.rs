use chrono::{DateTime, Utc};
use utoipa::IntoParams;
use uuid::Uuid;

/// Keyset position and size for a comment page on a post or photograph.
///
/// Pass back the previous page's `next_cursor` fields; omit both for the
/// first page. Comments are ordered oldest first.
#[derive(serde_derive::Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct CommentPageQuery {
    /// Creation time of the last comment already received.
    pub after_created_at: Option<DateTime<Utc>>,
    /// ID of the last comment already received.
    pub after_comment_id: Option<Uuid>,
    /// Page size from 1 to 100; defaults to 50.
    pub limit: Option<u16>,
}
