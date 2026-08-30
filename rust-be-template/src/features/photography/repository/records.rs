use chrono::{DateTime, Utc};
use diesel::{Insertable, Queryable, Selectable};
use uuid::Uuid;

use crate::{
    features::photography::{
        domain::{photograph::{NewPhotograph, Photograph}, social::{NewPhotographComment, PhotographComment}},
        repository::enums::DbPhotographContext,
    },
    schema::{photograph_comments, photographs},
};

#[derive(Queryable, Selectable)]
#[diesel(table_name = photographs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(super) struct PhotographRecord {
    photograph_id: Uuid, user_id: Uuid, photograph_shot_at: Option<DateTime<Utc>>,
    photograph_created_at: DateTime<Utc>, photograph_updated_at: DateTime<Utc>, photograph_image_type: i32,
    photograph_is_on_cloud: bool, photograph_link: String, photograph_comments: String, photograph_lat: f64,
    photograph_lon: f64, photograph_thumbnail_link: String, photograph_context: DbPhotographContext,
    photograph_view_count: i64, photograph_total_upvotes: i64, photograph_total_downvotes: i64,
}

impl PhotographRecord { pub(super) fn clone_author_id(&self) -> Uuid { self.user_id } }

impl From<PhotographRecord> for Photograph {
    fn from(row: PhotographRecord) -> Self { Self { photograph_id: row.photograph_id, user_id: row.user_id,
        photograph_shot_at: row.photograph_shot_at, photograph_created_at: row.photograph_created_at,
        photograph_updated_at: row.photograph_updated_at, photograph_image_type: row.photograph_image_type,
        photograph_is_on_cloud: row.photograph_is_on_cloud, photograph_link: row.photograph_link,
        photograph_comments: row.photograph_comments, photograph_lat: row.photograph_lat, photograph_lon: row.photograph_lon,
        photograph_thumbnail_link: row.photograph_thumbnail_link, photograph_context: row.photograph_context.into(),
        photograph_view_count: row.photograph_view_count, photograph_total_upvotes: row.photograph_total_upvotes,
        photograph_total_downvotes: row.photograph_total_downvotes } }
}

#[derive(Insertable)]
#[diesel(table_name = photographs)]
pub(super) struct NewPhotographRecord {
    user_id: Uuid, photograph_shot_at: Option<DateTime<Utc>>, photograph_image_type: i32,
    photograph_context: DbPhotographContext, photograph_is_on_cloud: bool, photograph_link: String,
    photograph_comments: String, photograph_lat: f64, photograph_lon: f64, photograph_thumbnail_link: String,
}

impl From<NewPhotograph> for NewPhotographRecord {
    fn from(value: NewPhotograph) -> Self { Self { user_id: value.user_id, photograph_shot_at: value.photograph_shot_at,
        photograph_image_type: value.photograph_image_type, photograph_context: value.photograph_context.into(),
        photograph_is_on_cloud: value.photograph_is_on_cloud, photograph_link: value.photograph_link,
        photograph_comments: value.photograph_comments, photograph_lat: value.photograph_lat, photograph_lon: value.photograph_lon,
        photograph_thumbnail_link: value.photograph_thumbnail_link } }
}

#[derive(Clone, Queryable, Selectable)]
#[diesel(table_name = photograph_comments)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(super) struct PhotographCommentRecord {
    photograph_comment_id: Uuid, photograph_id: Uuid, user_id: Uuid, photograph_comment_content: String,
    photograph_comment_created_at: DateTime<Utc>, photograph_comment_updated_at: Option<DateTime<Utc>>,
    parent_photograph_comment_id: Option<Uuid>, photograph_comment_total_upvotes: i64, photograph_comment_total_downvotes: i64,
}

impl From<PhotographCommentRecord> for PhotographComment {
    fn from(row: PhotographCommentRecord) -> Self { Self { photograph_comment_id: row.photograph_comment_id,
        photograph_id: row.photograph_id, user_id: row.user_id, photograph_comment_content: row.photograph_comment_content,
        photograph_comment_created_at: row.photograph_comment_created_at, photograph_comment_updated_at: row.photograph_comment_updated_at,
        parent_photograph_comment_id: row.parent_photograph_comment_id, photograph_comment_total_upvotes: row.photograph_comment_total_upvotes,
        photograph_comment_total_downvotes: row.photograph_comment_total_downvotes } }
}

#[derive(Insertable)]
#[diesel(table_name = photograph_comments)]
pub(super) struct NewPhotographCommentRecord {
    photograph_id: Uuid, user_id: Uuid, photograph_comment_content: String, parent_photograph_comment_id: Option<Uuid>,
}

impl From<NewPhotographComment> for NewPhotographCommentRecord {
    fn from(value: NewPhotographComment) -> Self { Self { photograph_id: value.photograph_id, user_id: value.user_id,
        photograph_comment_content: value.content, parent_photograph_comment_id: value.parent_comment_id } }
}
