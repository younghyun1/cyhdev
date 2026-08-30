use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use super::post::PostInfo;

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct CachedPostInfo {
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
    pub post_tags: Vec<String>,
}

impl CachedPostInfo {
    pub fn from_post_info_with_tags(post: PostInfo, post_tags: Vec<String>) -> Self {
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
            post_tags,
        }
    }
}
