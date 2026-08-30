use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use super::comment::CommentResponse;
use super::{cache::CachedPostInfo, vote::VoteState};
use crate::features::accounts::domain::{
    account::DELETED_USER_DISPLAY_NAME, public_author::PublicAuthor,
};

pub const MAX_BLOG_POST_TITLE_CHARS: usize = 256;
pub const MAX_BLOG_POST_MARKDOWN_CHARS: usize = 500_000;
pub const MAX_BLOG_POST_TAGS: usize = 32;
pub const MAX_BLOG_POST_TAG_CHARS: usize = 64;

#[derive(Clone, Serialize, ToSchema)]
pub struct Post {
    pub post_id: Uuid,
    pub user_id: Uuid,
    pub post_title: String,
    pub post_slug: String,
    pub post_content: String,
    pub post_summary: Option<String>,
    pub post_created_at: DateTime<Utc>,
    pub post_updated_at: DateTime<Utc>,
    pub post_published_at: Option<DateTime<Utc>>,
    pub post_is_published: bool,
    pub post_view_count: i64,
    pub post_share_count: i64,
    pub post_metadata: serde_json::Value,
    pub total_upvotes: i64,
    pub total_downvotes: i64,
}

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct PostInfo {
    pub post_id: Uuid,
    pub user_id: Uuid,
    pub post_title: String,
    pub post_slug: String,
    pub post_summary: Option<String>,
    pub post_created_at: DateTime<Utc>,
    pub post_updated_at: DateTime<Utc>,
    pub post_published_at: Option<DateTime<Utc>>,
    pub post_is_published: bool,
    pub post_view_count: i64,
    pub post_share_count: i64,
    pub total_upvotes: i64,
    pub total_downvotes: i64,
}

#[derive(Serialize, ToSchema)]
pub struct UserBadgeInfo {
    pub user_name: String,
    pub user_profile_picture_url: String,
    pub user_country_flag: Option<String>,
}

impl UserBadgeInfo {
    pub fn deleted() -> Self {
        Self {
            user_name: DELETED_USER_DISPLAY_NAME.to_owned(),
            user_profile_picture_url: String::new(),
            user_country_flag: None,
        }
    }

    pub fn from_public_author(author: &PublicAuthor, country_flag: Option<String>) -> Self {
        if author.is_deleted() {
            return Self::deleted();
        }
        Self {
            user_name: author.user_name().to_owned(),
            user_profile_picture_url: author.profile_picture_url().to_owned(),
            user_country_flag: country_flag,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct PostInfoWithVote {
    pub post_id: Uuid,
    pub user_id: Uuid,
    pub user_name: String,
    pub user_profile_picture_url: String,
    pub user_country_flag: Option<String>,
    pub post_title: String,
    pub post_slug: String,
    pub post_summary: Option<String>,
    pub post_created_at: DateTime<Utc>,
    pub post_updated_at: DateTime<Utc>,
    pub post_published_at: Option<DateTime<Utc>>,
    pub post_is_published: bool,
    pub post_view_count: i64,
    pub post_share_count: i64,
    pub total_upvotes: i64,
    pub total_downvotes: i64,
    pub post_tags: Vec<String>,
    pub vote_state: VoteState,
}

impl PostInfoWithVote {
    pub fn from_cached_info_with_vote(
        cached: CachedPostInfo,
        vote_state: VoteState,
        public_user_id: Uuid,
        badge: UserBadgeInfo,
    ) -> Self {
        Self {
            post_id: cached.post_id,
            user_id: public_user_id,
            user_name: badge.user_name,
            user_profile_picture_url: badge.user_profile_picture_url,
            user_country_flag: badge.user_country_flag,
            post_title: cached.post_title,
            post_slug: cached.post_slug,
            post_summary: cached.post_summary,
            post_created_at: cached.post_created_at,
            post_updated_at: cached.post_updated_at,
            post_published_at: cached.post_published_at,
            post_is_published: cached.post_is_published,
            post_view_count: cached.post_view_count,
            post_share_count: cached.post_share_count,
            total_upvotes: cached.total_upvotes,
            total_downvotes: cached.total_downvotes,
            post_tags: cached.post_tags,
            vote_state,
        }
    }
}

impl From<Post> for PostInfo {
    fn from(post: Post) -> Self {
        Self {
            post_id: post.post_id,
            user_id: post.user_id,
            post_title: post.post_title,
            post_slug: post.post_slug,
            post_summary: post.post_summary,
            post_created_at: post.post_created_at,
            post_updated_at: post.post_updated_at,
            post_published_at: post.post_published_at,
            post_is_published: post.post_is_published,
            post_view_count: post.post_view_count,
            post_share_count: post.post_share_count,
            total_upvotes: post.total_upvotes,
            total_downvotes: post.total_downvotes,
        }
    }
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct PostMetadata {}

pub enum PostLookup {
    Id(Uuid),
    Slug(String),
}

pub struct SavePostInput {
    pub actor_user_id: Uuid,
    pub post_id: Option<Uuid>,
    pub title: String,
    pub markdown: String,
    pub tags: Vec<String>,
    pub published: bool,
    pub owner_required: bool,
}

pub struct SavePostCommand {
    pub post_id: Option<Uuid>,
    pub actor_user_id: Uuid,
    pub title: String,
    pub slug: String,
    pub rendered_content: String,
    pub markdown_content: String,
    pub tags: Vec<String>,
    pub published: bool,
    pub owner_required: bool,
}

pub struct ReadPostResult {
    pub post: Post,
    pub post_tags: Vec<String>,
    pub comments: Vec<CommentResponse>,
    pub vote_state: VoteState,
    pub user_badge_info: UserBadgeInfo,
}
