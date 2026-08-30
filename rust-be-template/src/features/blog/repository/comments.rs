use std::collections::HashMap;

use chrono::Utc;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    features::accounts::domain::public_author::PublicAuthor,
    persistence::public_authors::load_public_authors,
    schema::{comments, posts},
};

use super::super::{domain::comment::Comment, error::BlogError};
use super::{
    authority::{lock_active_user, require_owner_or_superuser},
    blog_repository::BlogRepository,
    records::{CommentRecord, NewCommentRecord},
};

pub const MAX_COMPATIBILITY_POST_COMMENTS: usize = 1_000;
const MAX_COMPATIBILITY_COMMENT_QUERY_ROWS: i64 = 1_001;

pub struct CommentList {
    pub comments: Vec<Comment>,
    pub truncated: bool,
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

    pub async fn comments_for_post(&self, post_id: Uuid) -> Result<CommentList, BlogError> {
        let mut connection = self.connection().await?;
        let mut comments = comments::table
            .filter(comments::post_id.eq(post_id))
            .order((
                comments::comment_created_at.asc(),
                comments::comment_id.asc(),
            ))
            .select(CommentRecord::as_select())
            .limit(MAX_COMPATIBILITY_COMMENT_QUERY_ROWS)
            .load::<CommentRecord>(&mut connection)
            .await
            .map_err(BlogError::Database)?
            .into_iter()
            .map(Comment::from)
            .collect::<Vec<_>>();
        let truncated = comments.len() > MAX_COMPATIBILITY_POST_COMMENTS;
        if truncated {
            comments.truncate(MAX_COMPATIBILITY_POST_COMMENTS);
        }
        Ok(CommentList {
            comments,
            truncated,
        })
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
                posts::table
                    .find(post_id)
                    .select(posts::post_id)
                    .first::<Uuid>(&mut *connection)
                    .await
                    .optional()?
                    .map(|_| ())
                    .ok_or(BlogError::PostNotFound)?;
                if let Some(parent_comment_id) = parent_comment_id {
                    let parent_post_id = comments::table
                        .find(parent_comment_id)
                        .select(comments::post_id)
                        .first::<Uuid>(&mut *connection)
                        .await
                        .optional()?
                        .ok_or(BlogError::CommentNotFound)?;
                    if parent_post_id != post_id {
                        return Err(BlogError::InvalidInput);
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
                let owner_id = comments::table
                    .find(comment_id)
                    .select(comments::user_id)
                    .for_update()
                    .first::<Uuid>(&mut *connection)
                    .await
                    .optional()?
                    .ok_or(BlogError::CommentNotFound)?;
                require_owner_or_superuser(connection, requester_id, owner_id).await?;
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

    pub async fn delete_comment(
        &self,
        requester_id: Uuid,
        comment_id: Uuid,
    ) -> Result<(), BlogError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<(), BlogError, _>(async move |connection| {
                let owner_id = comments::table
                    .find(comment_id)
                    .select(comments::user_id)
                    .for_update()
                    .first::<Uuid>(&mut *connection)
                    .await
                    .optional()?
                    .ok_or(BlogError::CommentNotFound)?;
                require_owner_or_superuser(connection, requester_id, owner_id).await?;
                diesel::delete(comments::table.find(comment_id))
                    .execute(&mut *connection)
                    .await?;
                Ok(())
            })
            .await
    }
}
