use chrono::Utc;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::{AsyncConnection, RunQueryDsl};
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
    schema::{photograph_comment_votes, photograph_comments},
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
                if let Some(parent_id) = command.parent_comment_id {
                    let parent_photograph = photograph_comments::table
                        .filter(photograph_comments::photograph_comment_id.eq(parent_id))
                        .select(photograph_comments::photograph_id)
                        .first::<Uuid>(&mut *connection)
                        .await
                        .optional()?
                        .ok_or(PhotographyError::CommentNotFound)?;
                    if parent_photograph != command.photograph_id {
                        return Err(PhotographyError::InvalidInput);
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
                let author_id = photograph_comments::table
                    .filter(photograph_comments::photograph_comment_id.eq(comment_id))
                    .select(photograph_comments::user_id)
                    .first::<Uuid>(&mut *connection)
                    .await
                    .optional()?
                    .ok_or(PhotographyError::CommentNotFound)?;
                if author_id != requester_id {
                    lock_superuser(connection, requester_id).await?;
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

    pub async fn delete_comment(
        &self,
        requester_id: Uuid,
        comment_id: Uuid,
    ) -> Result<(), PhotographyError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<(), PhotographyError, _>(async move |connection| {
                lock_user(connection, requester_id).await?;
                let author_id = photograph_comments::table
                    .filter(photograph_comments::photograph_comment_id.eq(comment_id))
                    .select(photograph_comments::user_id)
                    .first::<Uuid>(&mut *connection)
                    .await
                    .optional()?
                    .ok_or(PhotographyError::CommentNotFound)?;
                if author_id != requester_id {
                    lock_superuser(connection, requester_id).await?;
                }
                diesel::delete(
                    photograph_comments::table
                        .filter(photograph_comments::photograph_comment_id.eq(comment_id)),
                )
                .execute(&mut *connection)
                .await?;
                Ok(())
            })
            .await
    }
}

fn vote_state(vote: Option<bool>) -> VoteState {
    match vote {
        Some(true) => VoteState::Upvoted,
        Some(false) => VoteState::Downvoted,
        None => VoteState::DidNotVote,
    }
}

async fn lock_user(
    connection: &mut diesel_async::AsyncPgConnection,
    user_id: Uuid,
) -> Result<(), PhotographyError> {
    map_authority(lock_active_user(connection, user_id).await)
}
async fn lock_superuser(
    connection: &mut diesel_async::AsyncPgConnection,
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
