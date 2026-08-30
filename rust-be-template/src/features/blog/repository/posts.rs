use std::collections::HashMap;

use chrono::Utc;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::schema::{post_tags, posts, tags};

use super::{
    authority::{has_current_blog_authority, lock_active_superuser, require_owner_or_superuser},
    blog_repository::BlogRepository,
    records::{NewPostRecord, NewPostTagRecord, NewTagRecord, PostRecord},
};
use super::super::{domain::post::{Post, PostLookup, SavePostCommand}, error::BlogError};

impl BlogRepository {
    pub async fn save_post(&self, command: SavePostCommand) -> Result<Post, BlogError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<Post, BlogError, _>(async move |connection| {
                lock_active_superuser(connection, command.actor_user_id).await?;
                let now = Utc::now();
                let metadata = serde_json::json!({"markdown_content": command.markdown_content});
                let post = match command.post_id {
                    Some(post_id) => {
                        let published_at = if command.owner_required {
                            posts::table
                                .find(post_id)
                                .filter(posts::user_id.eq(command.actor_user_id))
                                .select(posts::post_published_at)
                                .for_update()
                                .first::<Option<chrono::DateTime<Utc>>>(&mut *connection)
                                .await
                                .optional()?
                        } else {
                            posts::table
                                .find(post_id)
                                .select(posts::post_published_at)
                                .for_update()
                                .first::<Option<chrono::DateTime<Utc>>>(&mut *connection)
                                .await
                                .optional()?
                        }
                        .ok_or(BlogError::PostNotFound)?;
                        diesel::update(posts::table.find(post_id))
                            .set((
                                posts::post_title.eq(&command.title),
                                posts::post_slug.eq(&command.slug),
                                posts::post_content.eq(&command.rendered_content),
                                posts::post_is_published.eq(command.published),
                                posts::post_published_at.eq(if command.published {
                                    published_at.or(Some(now))
                                } else {
                                    None
                                }),
                                posts::post_updated_at.eq(now),
                                posts::post_metadata.eq(&metadata),
                            ))
                            .returning(PostRecord::as_returning())
                            .get_result::<PostRecord>(&mut *connection)
                            .await
                            .map_err(classify_write_error)?
                    }
                    None => diesel::insert_into(posts::table)
                        .values(NewPostRecord {
                            user_id: command.actor_user_id,
                            post_title: &command.title,
                            post_slug: &command.slug,
                            post_content: &command.rendered_content,
                            post_published_at: command.published.then_some(now),
                            post_is_published: command.published,
                            post_metadata: &metadata,
                        })
                        .returning(PostRecord::as_returning())
                        .get_result::<PostRecord>(&mut *connection)
                        .await
                        .map_err(classify_write_error)?,
                };
                replace_tags(connection, post.id(), &command.tags).await?;
                Ok(Post::from(post))
            })
            .await
    }

    pub async fn delete_post(
        &self,
        requester_id: Uuid,
        post_id: Uuid,
    ) -> Result<(), BlogError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<(), BlogError, _>(async move |connection| {
                let owner_id = posts::table
                    .find(post_id)
                    .select(posts::user_id)
                    .for_update()
                    .first::<Uuid>(&mut *connection)
                    .await
                    .optional()?
                    .ok_or(BlogError::PostNotFound)?;
                require_owner_or_superuser(connection, requester_id, owner_id).await?;
                diesel::delete(posts::table.find(post_id))
                    .execute(&mut *connection)
                    .await?;
                Ok(())
            })
            .await
    }

    pub async fn resolve_post_id(&self, lookup: &PostLookup) -> Result<Option<Uuid>, BlogError> {
        match lookup {
            PostLookup::Id(post_id) => Ok(Some(*post_id)),
            PostLookup::Slug(slug) => {
                let mut connection = self.connection().await?;
                posts::table
                    .filter(posts::post_slug.eq(slug))
                    .select(posts::post_id)
                    .first::<Uuid>(&mut connection)
                    .await
                    .optional()
                    .map_err(BlogError::Database)
            }
        }
    }

    pub async fn read_post(
        &self,
        post_id: Uuid,
        viewer_id: Option<Uuid>,
    ) -> Result<Post, BlogError> {
        let mut connection = self.connection().await?;
        let include_unpublished = has_current_blog_authority(&mut connection, viewer_id).await?;
        let record = if include_unpublished {
            posts::table.find(post_id)
                .select(PostRecord::as_select())
                .first::<PostRecord>(&mut connection)
                .await
        } else {
            posts::table
                .filter(posts::post_id.eq(post_id))
                .filter(posts::post_is_published.eq(true))
                .select(PostRecord::as_select())
                .first::<PostRecord>(&mut connection)
                .await
        };
        record
            .optional()?
            .map(Post::from)
            .ok_or(BlogError::PostNotFound)
    }

    pub async fn increment_post_view(&self, post_id: Uuid) -> Result<i64, BlogError> {
        let mut connection = self.connection().await?;
        diesel::update(posts::table.find(post_id))
            .set(posts::post_view_count.eq(posts::post_view_count + 1_i64))
            .returning(posts::post_view_count)
            .get_result::<i64>(&mut connection)
            .await
            .map_err(BlogError::Database)
    }

    pub async fn tags_for_post(&self, post_id: Uuid) -> Result<Vec<String>, BlogError> {
        let mut connection = self.connection().await?;
        post_tags::table
            .inner_join(tags::table)
            .filter(post_tags::post_id.eq(post_id))
            .select(tags::tag_name)
            .load(&mut connection)
            .await
            .map_err(BlogError::Database)
    }
}

async fn replace_tags(
    connection: &mut diesel_async::AsyncPgConnection,
    post_id: Uuid,
    requested_tags: &[String],
) -> Result<(), BlogError> {
    diesel::delete(post_tags::table.filter(post_tags::post_id.eq(post_id)))
        .execute(&mut *connection)
        .await?;
    if requested_tags.is_empty() {
        return Ok(());
    }
    let rows = requested_tags
        .iter()
        .map(|tag| NewTagRecord { tag_name: tag })
        .collect::<Vec<_>>();
    diesel::insert_into(tags::table)
        .values(rows)
        .on_conflict(tags::tag_name)
        .do_nothing()
        .execute(&mut *connection)
        .await?;
    let tag_ids = tags::table
        .filter(tags::tag_name.eq_any(requested_tags))
        .select((tags::tag_id, tags::tag_name))
        .load::<(i16, String)>(&mut *connection)
        .await?
        .into_iter()
        .map(|(id, name)| (name, id))
        .collect::<HashMap<_, _>>();
    let links = requested_tags
        .iter()
        .map(|tag| {
            tag_ids
                .get(tag)
                .copied()
                .map(|tag_id| NewPostTagRecord { post_id, tag_id })
                .ok_or(BlogError::Invariant("persisted post tag was not readable"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    diesel::insert_into(post_tags::table)
        .values(links)
        .execute(&mut *connection)
        .await?;
    Ok(())
}

fn classify_write_error(error: diesel::result::Error) -> BlogError {
    match &error {
        diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::UniqueViolation,
            _,
        ) => BlogError::DuplicateTitle,
        _ => BlogError::Database(error),
    }
}
