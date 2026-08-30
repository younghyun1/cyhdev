//! Persistence-independent photograph comment and vote values.

use chrono::{DateTime, Utc};
use serde_derive::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::features::accounts::domain::public_author::PublicAuthor;
use crate::features::blog::domain::{post::UserBadgeInfo, vote::VoteState};

/// Maximum number of Unicode scalar values accepted in one photograph comment.
pub const MAX_PHOTOGRAPH_COMMENT_CHARS: usize = 4_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhotographCommentBodyError {
    Empty,
    TooLong,
}

/// A non-empty, bounded comment body validated before reaching persistence.
pub struct PhotographCommentBody(String);

impl PhotographCommentBody {
    pub fn parse(content: String) -> Result<Self, PhotographCommentBodyError> {
        if content.trim().is_empty() {
            return Err(PhotographCommentBodyError::Empty);
        }
        if content.chars().count() > MAX_PHOTOGRAPH_COMMENT_CHARS {
            return Err(PhotographCommentBodyError::TooLong);
        }
        Ok(Self(content))
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

#[derive(Clone, Serialize, ToSchema)]
pub struct PhotographComment {
    pub photograph_comment_id: Uuid,
    pub photograph_id: Uuid,
    pub user_id: Uuid,
    pub photograph_comment_content: String,
    pub photograph_comment_created_at: DateTime<Utc>,
    pub photograph_comment_updated_at: Option<DateTime<Utc>>,
    pub parent_photograph_comment_id: Option<Uuid>,
    pub photograph_comment_total_upvotes: i64,
    pub photograph_comment_total_downvotes: i64,
}

pub struct NewPhotographComment {
    pub photograph_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub parent_comment_id: Option<Uuid>,
}

#[derive(Clone, Serialize, ToSchema)]
pub struct PhotographCommentResponse {
    pub photograph_comment_id: Uuid,
    pub photograph_id: Uuid,
    pub user_id: Uuid,
    pub photograph_comment_content: String,
    pub photograph_comment_created_at: DateTime<Utc>,
    pub photograph_comment_updated_at: Option<DateTime<Utc>>,
    pub parent_photograph_comment_id: Option<Uuid>,
    pub photograph_comment_total_upvotes: i64,
    pub photograph_comment_total_downvotes: i64,
    pub vote_state: VoteState,
    pub user_name: String,
    pub user_profile_picture_url: String,
    pub user_country_flag: Option<String>,
}

impl PhotographCommentResponse {
    pub fn from_comment_votestate_and_badge_info(
        comment: PhotographComment,
        vote_state: VoteState,
        public_user_id: Uuid,
        badge: UserBadgeInfo,
    ) -> Self {
        Self {
            photograph_comment_id: comment.photograph_comment_id,
            photograph_id: comment.photograph_id,
            user_id: public_user_id,
            photograph_comment_content: comment.photograph_comment_content,
            photograph_comment_created_at: comment.photograph_comment_created_at,
            photograph_comment_updated_at: comment.photograph_comment_updated_at,
            parent_photograph_comment_id: comment.parent_photograph_comment_id,
            photograph_comment_total_upvotes: comment.photograph_comment_total_upvotes,
            photograph_comment_total_downvotes: comment.photograph_comment_total_downvotes,
            vote_state,
            user_name: badge.user_name,
            user_profile_picture_url: badge.user_profile_picture_url,
            user_country_flag: badge.user_country_flag,
        }
    }
}

#[derive(Clone, Copy)]
pub struct VoteCounts {
    pub upvote_count: i64,
    pub downvote_count: i64,
}

pub struct CommentMutation {
    pub comment: PhotographComment,
    pub vote_state: VoteState,
}

pub struct CommentPresentation {
    pub comment: PhotographComment,
    pub vote_state: VoteState,
    pub author: PublicAuthor,
}
