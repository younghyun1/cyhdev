use utoipa::ToSchema;

use crate::dto::responses::blog::comment_page_response::CommentCursorResponse;

use crate::features::blog::domain::{post::UserBadgeInfo, vote::VoteState};
use crate::features::photography::domain::{
    photograph::Photograph, social::PhotographCommentResponse,
};

/// Detail response for a single photograph: the row (incl. denormalized view +
/// vote counts), the caller's vote state, the first oldest-first comment page,
/// and the photograph author's badge. Comments are threaded client-side via
/// `parent_photograph_comment_id`; later pages come from the comments endpoint.
#[derive(serde_derive::Serialize, ToSchema)]
pub struct ReadPhotographResponse {
    pub photograph: Photograph,
    pub vote_state: VoteState,
    pub comments: Vec<PhotographCommentResponse>,
    #[schema(required)]
    pub comments_next_cursor: Option<CommentCursorResponse>,
    pub user_badge_info: UserBadgeInfo,
}
