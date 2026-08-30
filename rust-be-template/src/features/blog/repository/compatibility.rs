//! Legacy integration-test insertables kept outside the blog domain.

use chrono::{DateTime, Utc};
use diesel::Insertable;
use uuid::Uuid;

use crate::schema::{comment_votes, post_votes, posts};

#[derive(Insertable)]
#[diesel(table_name = posts)]
pub struct NewPost<'a> {
    user_id: &'a Uuid,
    post_title: &'a str,
    post_slug: &'a str,
    post_content: &'a str,
    post_published_at: Option<DateTime<Utc>>,
    post_is_published: bool,
    post_metadata: &'a serde_json::Value,
}

impl<'a> NewPost<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        user_id: &'a Uuid,
        post_title: &'a str,
        post_slug: &'a str,
        post_content: &'a str,
        post_published_at: Option<DateTime<Utc>>,
        post_is_published: bool,
        post_metadata: &'a serde_json::Value,
    ) -> Self {
        Self {
            user_id,
            post_title,
            post_slug,
            post_content,
            post_published_at,
            post_is_published,
            post_metadata,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = post_votes)]
pub struct NewPostVote<'a> {
    post_id: &'a Uuid,
    user_id: &'a Uuid,
    is_upvote: bool,
}

impl<'a> NewPostVote<'a> {
    pub fn new(post_id: &'a Uuid, user_id: &'a Uuid, is_upvote: bool) -> Self {
        Self {
            post_id,
            user_id,
            is_upvote,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = comment_votes)]
pub struct NewCommentVote<'a> {
    comment_id: &'a Uuid,
    user_id: &'a Uuid,
    is_upvote: bool,
}

impl<'a> NewCommentVote<'a> {
    pub fn new(comment_id: &'a Uuid, user_id: &'a Uuid, is_upvote: bool) -> Self {
        Self {
            comment_id,
            user_id,
            is_upvote,
        }
    }
}
