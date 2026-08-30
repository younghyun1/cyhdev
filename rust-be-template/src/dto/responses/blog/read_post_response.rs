use utoipa::ToSchema;

use crate::features::blog::domain::{
    comment::CommentResponse,
    post::{Post, UserBadgeInfo},
    vote::VoteState,
};

#[derive(serde_derive::Serialize, ToSchema)]
pub struct ReadPostResponse {
    pub post: Post,
    pub post_tags: Vec<String>,
    pub comments: Vec<CommentResponse>,
    pub vote_state: VoteState,
    pub user_badge_info: UserBadgeInfo,
}
