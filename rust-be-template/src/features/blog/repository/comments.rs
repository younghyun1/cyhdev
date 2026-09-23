//! Blog comment writes: creation, edits, and tombstone deletion.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    features::accounts::domain::public_author::PublicAuthor,
    persistence::public_authors::load_public_authors,
    schema::{comments, posts},
};

use super::super::{domain::comment::Comment, error::BlogError};
use super::{
    authority::{lock_active_user, require_owner_or_superuser, require_visible_post},
    blog_repository::BlogRepository,
    records::{CommentRecord, NewCommentRecord},
};

/// Owner, deletion state, and post of a comment locked for mutation.
struct LockedComment {
    owner_id: Uuid,
    deleted_at: Option<DateTime<Utc>>,
}

impl BlogRepository {
    pub async fn authors_by_ids(
        &self,
        user_ids: &[Uuid],
    ) -> Result<HashMap<Uuid, PublicAuthor>, BlogError> {
        let mut connection = self.connection().await?;
        load_public_authors(&mut connection, user_ids)
            .await
            .map_err(BlogError::Database)
    }

    pub async fn insert_comment(
        &self,
        user_id: Uuid,
        post_id: Uuid,
        parent_comment_id: Option<Uuid>,
        content: &str,
    ) -> Result<Comment, BlogError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<Comment, BlogError, _>(async move |connection| {
                lock_active_user(connection, user_id).await?;
                let published = posts::table
                    .find(post_id)
                    .select(posts::post_is_published)
                    .first::<bool>(&mut *connection)
                    .await
                    .optional()?
                    .ok_or(BlogError::PostNotFound)?;
                require_visible_post(connection, user_id, published).await?;
                if let Some(parent_comment_id) = parent_comment_id {
                    let (parent_post_id, parent_deleted_at) = comments::table
                        .find(parent_comment_id)
                        .select((comments::post_id, comments::comment_deleted_at))
                        .first::<(Uuid, Option<DateTime<Utc>>)>(&mut *connection)
                        .await
                        .optional()?
                        .ok_or(BlogError::CommentNotFound)?;
                    if parent_post_id != post_id {
                        return Err(BlogError::InvalidInput);
                    }
                    // A tombstone keeps existing replies attached but takes no new ones.
                    if parent_deleted_at.is_some() {
                        return Err(BlogError::CommentNotFound);
                    }
                }
                diesel::insert_into(comments::table)
                    .values(NewCommentRecord {
                        post_id,
                        user_id,
                        comment_content: content,
                        parent_comment_id,
                    })
                    .returning(CommentRecord::as_returning())
                    .get_result::<CommentRecord>(&mut *connection)
                    .await
                    .map(Comment::from)
                    .map_err(BlogError::Database)
            })
            .await
    }

    pub async fn update_comment(
        &self,
        requester_id: Uuid,
        comment_id: Uuid,
        content: &str,
    ) -> Result<Comment, BlogError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<Comment, BlogError, _>(async move |connection| {
                let role = lock_active_user(connection, requester_id).await?;
                let locked = lock_comment(connection, comment_id).await?;
                require_owner_or_superuser(requester_id, role, locked.owner_id)?;
                if locked.deleted_at.is_some() {
                    return Err(BlogError::CommentNotFound);
                }
                diesel::update(comments::table.find(comment_id))
                    .set((
                        comments::comment_content.eq(content),
                        comments::comment_updated_at.eq(Utc::now()),
                    ))
                    .returning(CommentRecord::as_returning())
                    .get_result::<CommentRecord>(&mut *connection)
                    .await
                    .map(Comment::from)
                    .map_err(BlogError::Database)
            })
            .await
    }

    /// Replaces a comment with a tombstone so replies by other users survive.
    ///
    /// Repeating the request after deletion succeeds without further change.
    pub async fn delete_comment(
        &self,
        requester_id: Uuid,
        comment_id: Uuid,
    ) -> Result<(), BlogError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<(), BlogError, _>(async move |connection| {
                let role = lock_active_user(connection, requester_id).await?;
                let locked = lock_comment(connection, comment_id).await?;
                require_owner_or_superuser(requester_id, role, locked.owner_id)?;
                if locked.deleted_at.is_some() {
                    return Ok(());
                }
                diesel::update(comments::table.find(comment_id))
                    .set((
                        comments::comment_content.eq(""),
                        comments::comment_deleted_at.eq(Some(Utc::now())),
                    ))
                    .execute(&mut *connection)
                    .await?;
                Ok(())
            })
            .await
    }
}

async fn lock_comment(
    connection: &mut AsyncPgConnection,
    comment_id: Uuid,
) -> Result<LockedComment, BlogError> {
    comments::table
        .find(comment_id)
        .select((comments::user_id, comments::comment_deleted_at))
        .for_update()
        .first::<(Uuid, Option<DateTime<Utc>>)>(&mut *connection)
        .await
        .optional()?
        .map(|(owner_id, deleted_at)| LockedComment {
            owner_id,
            deleted_at,
        })
        .ok_or(BlogError::CommentNotFound)
}
