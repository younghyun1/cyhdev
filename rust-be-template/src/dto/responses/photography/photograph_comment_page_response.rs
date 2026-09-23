use serde_derive::Serialize;
use utoipa::ToSchema;

use crate::dto::responses::blog::comment_page_response::CommentCursorResponse;
use crate::features::photography::domain::social::PhotographCommentResponse;

/// One oldest-first page of a photograph's comments. Deleted comments appear
/// as tombstones with empty content so their replies stay attached.
#[derive(Serialize, ToSchema)]
pub struct PhotographCommentPageResponse {
    pub comments: Vec<PhotographCommentResponse>,
    #[schema(required)]
    pub next_cursor: Option<CommentCursorResponse>,
}
