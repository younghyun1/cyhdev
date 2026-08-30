use chrono::{DateTime, Utc};
use serde_derive::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use super::{post::UserBadgeInfo, vote::VoteState};

pub const MAX_BLOG_COMMENT_CHARS: usize = 4_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlogCommentBodyError {
    Empty,
    TooLong,
}

/// A non-empty comment body bounded by Unicode scalar values.
pub struct BlogCommentBody(String);

impl BlogCommentBody {
    pub fn parse(content: String) -> Result<Self, BlogCommentBodyError> {
        if content.trim().is_empty() {
            return Err(BlogCommentBodyError::Empty);
        }
        if content.chars().count() > MAX_BLOG_COMMENT_CHARS {
            return Err(BlogCommentBodyError::TooLong);
        }
        Ok(Self(content))
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

#[derive(Clone, Serialize, ToSchema)]
pub struct Comment {
    pub comment_id: Uuid,
    pub post_id: Uuid,
    pub user_id: Uuid,
    pub comment_content: String,
    pub comment_created_at: DateTime<Utc>,
    pub comment_updated_at: Option<DateTime<Utc>>,
    pub parent_comment_id: Option<Uuid>,
    pub total_upvotes: i64,
    pub total_downvotes: i64,
}

#[derive(Clone, Serialize, ToSchema)]
pub struct CommentResponse {
    pub comment_id: Uuid,
    pub post_id: Uuid,
    pub user_id: Uuid,
    pub comment_content: String,
    pub comment_created_at: DateTime<Utc>,
    pub comment_updated_at: Option<DateTime<Utc>>,
    pub parent_comment_id: Option<Uuid>,
    pub total_upvotes: i64,
    pub total_downvotes: i64,
    pub vote_state: VoteState,
    pub user_name: String,
    pub user_profile_picture_url: String,
    pub user_country_flag: Option<String>,
}

impl CommentResponse {
    pub fn from_comment_votestate_and_badge_info(
        comment: Comment,
        vote_state: VoteState,
        public_user_id: Uuid,
        badge: UserBadgeInfo,
    ) -> Self {
        Self {
            comment_id: comment.comment_id,
            post_id: comment.post_id,
            user_id: public_user_id,
            comment_content: comment.comment_content,
            comment_created_at: comment.comment_created_at,
            comment_updated_at: comment.comment_updated_at,
            parent_comment_id: comment.parent_comment_id,
            total_upvotes: comment.total_upvotes,
            total_downvotes: comment.total_downvotes,
            vote_state,
            user_name: badge.user_name,
            user_profile_picture_url: badge.user_profile_picture_url,
            user_country_flag: badge.user_country_flag,
        }
    }
}
