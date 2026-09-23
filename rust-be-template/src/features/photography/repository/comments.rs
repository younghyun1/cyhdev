//! Photograph comment writes: creation, edits, and tombstone deletion.

use chrono::{DateTime, Utc};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    features::accounts::domain::public_author::PublicAuthor,
    features::blog::domain::vote::VoteState,
    features::photography::{
        domain::social::{CommentMutation, NewPhotographComment, PhotographComment},
        error::PhotographyError,
        repository::{
            photography_repository::PhotographyRepository,
            records::{NewPhotographCommentRecord, PhotographCommentRecord},
        },
    },
    persistence::active_user::{ActiveUserWriteError, lock_active_superuser, lock_active_user},
    persistence::public_authors::load_public_authors,
    schema::{photograph_comment_votes, photograph_comments, photographs},
};

impl PhotographyRepository {
    pub async fn create_comment(
        &self,
        command: NewPhotographComment,
    ) -> Result<CommentMutation, PhotographyError> {
        let mut connection = self.connection().await?;
        let comment = connection
            .transaction::<PhotographComment, PhotographyError, _>(async move |connection| {
                lock_user(connection, command.user_id).await?;
                photographs::table
                    .filter(photographs::photograph_id.eq(command.photograph_id))
                    .select(photographs::photograph_id)
                    .first::<Uuid>(&mut *connection)
                    .await
                    .optional()?
                    .ok_or(PhotographyError::PhotographNotFound)?;
                if let Some(parent_id) = command.parent_comment_id {
                    let (parent_photograph, parent_deleted_at) = photograph_comments::table
                        .filter(photograph_comments::photograph_comment_id.eq(parent_id))
                        .select((
                            photograph_comments::photograph_id,
                            photograph_comments::photograph_comment_deleted_at,
                        ))
                        .first::<(Uuid, Option<DateTime<Utc>>)>(&mut *connection)
                        .await
                        .optional()?
                        .ok_or(PhotographyError::CommentNotFound)?;
                    if parent_photograph != command.photograph_id {
                        return Err(PhotographyError::InvalidInput);
                    }
                    // A tombstone keeps existing replies attached but takes no new ones.
                    if parent_deleted_at.is_some() {
                        return Err(PhotographyError::CommentNotFound);
                    }
                }
                let record = diesel::insert_into(photograph_comments::table)
                    .values(NewPhotographCommentRecord::from(command))
                    .returning(PhotographCommentRecord::as_returning())
                    .get_result::<PhotographCommentRecord>(&mut *connection)
                    .await?;
                Ok(record.into())
            })
            .await?;
        Ok(CommentMutation {
            comment,
            vote_state: VoteState::DidNotVote,
        })
    }

    pub async fn update_comment(
        &self,
        requester_id: Uuid,
        comment_id: Uuid,
        content: String,
    ) -> Result<CommentMutation, PhotographyError> {
        let mut connection = self.connection().await?;
        let mutation = connection
            .transaction::<CommentMutation, PhotographyError, _>(async move |connection| {
                lock_user(connection, requester_id).await?;
                let (author_id, deleted_at) = lock_comment(connection, comment_id).await?;
                if author_id != requester_id {
                    lock_superuser(connection, requester_id).await?;
                }
                if deleted_at.is_some() {
                    return Err(PhotographyError::CommentNotFound);
                }
                let record = diesel::update(
                    photograph_comments::table
                        .filter(photograph_comments::photograph_comment_id.eq(comment_id)),
                )
                .set((
                    photograph_comments::photograph_comment_content.eq(content),
                    photograph_comments::photograph_comment_updated_at.eq(Utc::now()),
                ))
                .returning(PhotographCommentRecord::as_returning())
                .get_result::<PhotographCommentRecord>(&mut *connection)
                .await?;
                let vote = photograph_comment_votes::table
                    .filter(photograph_comment_votes::photograph_comment_id.eq(comment_id))
                    .filter(photograph_comment_votes::user_id.eq(requester_id))
                    .select(photograph_comment_votes::is_upvote)
                    .first::<bool>(&mut *connection)
                    .await
                    .optional()?;
                Ok(CommentMutation {
                    comment: record.into(),
                    vote_state: vote_state(vote),
                })
            })
            .await?;
        Ok(mutation)
    }

    pub async fn comment_author(&self, user_id: Uuid) -> Result<PublicAuthor, PhotographyError> {
        let mut connection = self.connection().await?;
        let authors = load_public_authors(&mut connection, &[user_id]).await?;
        Ok(authors
            .get(&user_id)
            .cloned()
            .unwrap_or_else(PublicAuthor::deleted))
    }

    /// Replaces a comment with a tombstone so replies by other users survive.
    ///
    /// Repeating the request after deletion succeeds without further change.
    pub async fn delete_comment(
        &self,
        requester_id: Uuid,
        comment_id: Uuid,
    ) -> Result<(), PhotographyError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<(), PhotographyError, _>(async move |connection| {
                lock_user(connection, requester_id).await?;
                let (author_id, deleted_at) = lock_comment(connection, comment_id).await?;
                if author_id != requester_id {
                    lock_superuser(connection, requester_id).await?;
                }
                if deleted_at.is_some() {
                    return Ok(());
                }
                diesel::update(
                    photograph_comments::table
                        .filter(photograph_comments::photograph_comment_id.eq(comment_id)),
                )
                .set((
                    photograph_comments::photograph_comment_content.eq(""),
                    photograph_comments::photograph_comment_deleted_at.eq(Some(Utc::now())),
                ))
                .execute(&mut *connection)
                .await?;
                Ok(())
            })
            .await
    }
}

/// Locks a comment after its actor, returning its author and deletion time.
async fn lock_comment(
    connection: &mut AsyncPgConnection,
    comment_id: Uuid,
) -> Result<(Uuid, Option<DateTime<Utc>>), PhotographyError> {
    photograph_comments::table
        .filter(photograph_comments::photograph_comment_id.eq(comment_id))
        .select((
            photograph_comments::user_id,
            photograph_comments::photograph_comment_deleted_at,
        ))
        .for_update()
        .first::<(Uuid, Option<DateTime<Utc>>)>(&mut *connection)
        .await
        .optional()?
        .ok_or(PhotographyError::CommentNotFound)
}

fn vote_state(vote: Option<bool>) -> VoteState {
    match vote {
        Some(true) => VoteState::Upvoted,
        Some(false) => VoteState::Downvoted,
        None => VoteState::DidNotVote,
    }
}

async fn lock_user(
    connection: &mut AsyncPgConnection,
    user_id: Uuid,
) -> Result<(), PhotographyError> {
    map_authority(lock_active_user(connection, user_id).await)
}
async fn lock_superuser(
    connection: &mut AsyncPgConnection,
    user_id: Uuid,
) -> Result<(), PhotographyError> {
    map_authority(lock_active_superuser(connection, user_id).await)
}
fn map_authority(result: Result<(), ActiveUserWriteError>) -> Result<(), PhotographyError> {
    result.map_err(|error| match error {
        ActiveUserWriteError::Inactive => PhotographyError::InactiveAccount,
        ActiveUserWriteError::Denied => PhotographyError::Forbidden,
        ActiveUserWriteError::TargetNotFound => PhotographyError::CommentNotFound,
        ActiveUserWriteError::Database(error) => PhotographyError::Query(error),
    })
}
