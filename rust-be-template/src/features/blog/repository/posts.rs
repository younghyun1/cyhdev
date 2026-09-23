use std::collections::HashMap;

use chrono::Utc;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::schema::{post_tags, posts, tags};

use super::super::{
    domain::post::{Post, PostLookup, SavePostCommand},
    error::BlogError,
};
use super::{
    authority::{
        has_current_blog_authority, lock_active_superuser, lock_active_user,
        require_owner_or_superuser,
    },
    blog_repository::BlogRepository,
    records::{NewPostRecord, NewPostTagRecord, NewTagRecord, PostRecord},
};

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

    pub async fn delete_post(&self, requester_id: Uuid, post_id: Uuid) -> Result<(), BlogError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<(), BlogError, _>(async move |connection| {
                // Actor first, then content: the order every blog write uses.
                let role = lock_active_user(connection, requester_id).await?;
                let owner_id = posts::table
                    .find(post_id)
                    .select(posts::user_id)
                    .for_update()
                    .first::<Uuid>(&mut *connection)
                    .await
                    .optional()?
                    .ok_or(BlogError::PostNotFound)?;
                require_owner_or_superuser(requester_id, role, owner_id)?;
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

    /// Reads a post and, when the caller lacks a cached copy, its tags, using
    /// one pool checkout. Drafts are visible only to blog managers.
    pub async fn read_post(
        &self,
        post_id: Uuid,
        viewer_id: Option<Uuid>,
        load_tags: bool,
    ) -> Result<PostRead, BlogError> {
        let mut connection = self.connection().await?;
        let include_unpublished = has_current_blog_authority(&mut connection, viewer_id).await?;
        let mut query = posts::table
            .filter(posts::post_id.eq(post_id))
            .select(PostRecord::as_select())
            .into_boxed();
        if !include_unpublished {
            query = query.filter(posts::post_is_published.eq(true));
        }
        let post = query
            .first::<PostRecord>(&mut connection)
            .await
            .optional()?
            .map(Post::from)
            .ok_or(BlogError::PostNotFound)?;
        let tags = if load_tags {
            Some(
                post_tags::table
                    .inner_join(tags::table)
                    .filter(post_tags::post_id.eq(post_id))
                    .select(tags::tag_name)
                    .load::<String>(&mut connection)
                    .await?,
            )
        } else {
            None
        };
        Ok(PostRead { post, tags })
    }
}

/// A post detail read; `tags` is present only when the caller requested them.
pub struct PostRead {
    pub post: Post,
    pub tags: Option<Vec<String>>,
}

/// Replaces a post's tag links, inserting only tag names that do not exist.
///
/// `tags.tag_id` is an identity column, and PostgreSQL consumes an identity
/// value for every row an insert attempts, even one `ON CONFLICT DO NOTHING`
/// skips. Reading existing names first keeps re-saving a post from burning
/// identifiers; the conflict clause only absorbs a concurrent insert of the
/// same new name.
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
    let mut tag_ids = tag_ids_by_name(connection, requested_tags).await?;
    let missing = requested_tags
        .iter()
        .filter(|tag| !tag_ids.contains_key(tag.as_str()))
        .map(|tag| NewTagRecord { tag_name: tag })
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        diesel::insert_into(tags::table)
            .values(missing)
            .on_conflict(tags::tag_name)
            .do_nothing()
            .execute(&mut *connection)
            .await?;
        tag_ids = tag_ids_by_name(connection, requested_tags).await?;
    }
    let links = requested_tags
        .iter()
        .map(|tag| {
            tag_ids
                .get(tag.as_str())
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

async fn tag_ids_by_name(
    connection: &mut diesel_async::AsyncPgConnection,
    names: &[String],
) -> Result<HashMap<String, i32>, BlogError> {
    Ok(tags::table
        .filter(tags::tag_name.eq_any(names))
        .select((tags::tag_name, tags::tag_id))
        .load::<(String, i32)>(&mut *connection)
        .await?
        .into_iter()
        .collect())
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
