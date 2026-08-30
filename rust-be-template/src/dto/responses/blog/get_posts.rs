use serde_derive::Serialize;
use utoipa::ToSchema;

use crate::features::blog::domain::post::PostInfoWithVote;

#[derive(Serialize, ToSchema)]
pub struct GetPostsResponse {
    pub posts: Vec<PostInfoWithVote>,
    pub available_pages: usize,
}
