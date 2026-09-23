use utoipa::ToSchema;

use super::comment_page_response::CommentCursorResponse;
use crate::features::blog::domain::{
    comment::CommentResponse,
    post::{Post, UserBadgeInfo},
    vote::VoteState,
};

#[derive(serde_derive::Serialize, ToSchema)]
pub struct ReadPostResponse {
    pub post: Post,
    pub post_tags: Vec<String>,
    /// First oldest-first comment page; later pages come from the comments endpoint.
    pub comments: Vec<CommentResponse>,
    #[schema(required)]
    pub comments_next_cursor: Option<CommentCursorResponse>,
    pub vote_state: VoteState,
    pub user_badge_info: UserBadgeInfo,
}
