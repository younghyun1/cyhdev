use chrono::{DateTime, Utc};
use diesel::{Insertable, Queryable, Selectable};
use uuid::Uuid;

use crate::schema::{comments, post_tags, posts, tags};

use super::super::domain::{
    comment::Comment,
    post::{Post, PostInfo},
};

#[derive(Queryable, Selectable)]
#[diesel(table_name = posts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(super) struct PostRecord {
    post_id: Uuid,
    user_id: Uuid,
    post_title: String,
    post_slug: String,
    post_content: String,
    post_summary: Option<String>,
    post_created_at: DateTime<Utc>,
    post_updated_at: DateTime<Utc>,
    post_published_at: Option<DateTime<Utc>>,
    post_is_published: bool,
    post_view_count: i64,
    post_share_count: i64,
    post_metadata: serde_json::Value,
    total_upvotes: i64,
    total_downvotes: i64,
}

impl PostRecord {
    pub(super) const fn id(&self) -> Uuid {
        self.post_id
    }
}

impl From<PostRecord> for Post {
    fn from(row: PostRecord) -> Self {
        Self {
            post_id: row.post_id,
            user_id: row.user_id,
            post_title: row.post_title,
            post_slug: row.post_slug,
            post_content: row.post_content,
            post_summary: row.post_summary,
            post_created_at: row.post_created_at,
            post_updated_at: row.post_updated_at,
            post_published_at: row.post_published_at,
            post_is_published: row.post_is_published,
            post_view_count: row.post_view_count,
            post_share_count: row.post_share_count,
            post_metadata: row.post_metadata,
            total_upvotes: row.total_upvotes,
            total_downvotes: row.total_downvotes,
        }
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = posts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(super) struct PostInfoRecord {
    post_id: Uuid,
    user_id: Uuid,
    post_title: String,
    post_slug: String,
    post_summary: Option<String>,
    post_created_at: DateTime<Utc>,
    post_updated_at: DateTime<Utc>,
    post_published_at: Option<DateTime<Utc>>,
    post_is_published: bool,
    post_view_count: i64,
    post_share_count: i64,
    total_upvotes: i64,
    total_downvotes: i64,
}

impl From<PostInfoRecord> for PostInfo {
    fn from(row: PostInfoRecord) -> Self {
        Self {
            post_id: row.post_id,
            user_id: row.user_id,
            post_title: row.post_title,
            post_slug: row.post_slug,
            post_summary: row.post_summary,
            post_created_at: row.post_created_at,
            post_updated_at: row.post_updated_at,
            post_published_at: row.post_published_at,
            post_is_published: row.post_is_published,
            post_view_count: row.post_view_count,
            post_share_count: row.post_share_count,
            total_upvotes: row.total_upvotes,
            total_downvotes: row.total_downvotes,
        }
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = comments)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(super) struct CommentRecord {
    comment_id: Uuid,
    post_id: Uuid,
    user_id: Uuid,
    comment_content: String,
    comment_created_at: DateTime<Utc>,
    comment_updated_at: Option<DateTime<Utc>>,
    parent_comment_id: Option<Uuid>,
    total_upvotes: i64,
    total_downvotes: i64,
}

impl From<CommentRecord> for Comment {
    fn from(row: CommentRecord) -> Self {
        Self {
            comment_id: row.comment_id,
            post_id: row.post_id,
            user_id: row.user_id,
            comment_content: row.comment_content,
            comment_created_at: row.comment_created_at,
            comment_updated_at: row.comment_updated_at,
            parent_comment_id: row.parent_comment_id,
            total_upvotes: row.total_upvotes,
            total_downvotes: row.total_downvotes,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = posts)]
pub(super) struct NewPostRecord<'a> {
    pub user_id: Uuid,
    pub post_title: &'a str,
    pub post_slug: &'a str,
    pub post_content: &'a str,
    pub post_published_at: Option<DateTime<Utc>>,
    pub post_is_published: bool,
    pub post_metadata: &'a serde_json::Value,
}

#[derive(Insertable)]
#[diesel(table_name = comments)]
pub(super) struct NewCommentRecord<'a> {
    pub post_id: Uuid,
    pub user_id: Uuid,
    pub comment_content: &'a str,
    pub parent_comment_id: Option<Uuid>,
}

#[derive(Insertable)]
#[diesel(table_name = tags)]
pub(super) struct NewTagRecord<'a> {
    pub tag_name: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = post_tags)]
pub(super) struct NewPostTagRecord {
    pub post_id: Uuid,
    pub tag_id: i16,
}
