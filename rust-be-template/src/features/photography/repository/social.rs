use diesel::{
    BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, dsl::count_star,
};
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    features::photography::{
        domain::social::VoteCounts, error::PhotographyError,
        repository::photography_repository::PhotographyRepository,
    },
    persistence::active_user::{ActiveUserWriteError, lock_active_user},
    schema::{photograph_comment_votes, photograph_comments, photograph_votes, photographs},
};

impl PhotographyRepository {
    pub async fn vote_photograph(
        &self,
        user_id: Uuid,
        photograph_id: Uuid,
        is_upvote: bool,
    ) -> Result<VoteCounts, PhotographyError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<VoteCounts, PhotographyError, _>(async move |connection| {
                lock_actor(connection, user_id).await?;
                lock_photograph(connection, photograph_id).await?;
                diesel::insert_into(photograph_votes::table)
                    .values((
                        photograph_votes::photograph_id.eq(photograph_id),
                        photograph_votes::user_id.eq(user_id),
                        photograph_votes::is_upvote.eq(is_upvote),
                    ))
                    .on_conflict((photograph_votes::photograph_id, photograph_votes::user_id))
                    .do_update()
                    .set(photograph_votes::is_upvote.eq(is_upvote))
                    .execute(&mut *connection)
                    .await?;
                let counts = photograph_vote_counts(connection, photograph_id).await?;
                diesel::update(
                    photographs::table.filter(photographs::photograph_id.eq(photograph_id)),
                )
                .set((
                    photographs::photograph_total_upvotes.eq(counts.upvote_count),
                    photographs::photograph_total_downvotes.eq(counts.downvote_count),
                ))
                .execute(&mut *connection)
                .await?;
                Ok(counts)
            })
            .await
    }

    pub async fn rescind_photograph_vote(
        &self,
        user_id: Uuid,
        photograph_id: Uuid,
    ) -> Result<VoteCounts, PhotographyError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<VoteCounts, PhotographyError, _>(async move |connection| {
                lock_actor(connection, user_id).await?;
                lock_photograph(connection, photograph_id).await?;
                let affected = diesel::delete(
                    photograph_votes::table.filter(
                        photograph_votes::photograph_id
                            .eq(photograph_id)
                            .and(photograph_votes::user_id.eq(user_id)),
                    ),
                )
                .execute(&mut *connection)
                .await?;
                if affected == 0 {
                    return Err(PhotographyError::VoteNotFound);
                }
                let counts = photograph_vote_counts(connection, photograph_id).await?;
                diesel::update(
                    photographs::table.filter(photographs::photograph_id.eq(photograph_id)),
                )
                .set((
                    photographs::photograph_total_upvotes.eq(counts.upvote_count),
                    photographs::photograph_total_downvotes.eq(counts.downvote_count),
                ))
                .execute(&mut *connection)
                .await?;
                Ok(counts)
            })
            .await
    }

    pub async fn vote_comment(
        &self,
        user_id: Uuid,
        comment_id: Uuid,
        is_upvote: bool,
    ) -> Result<VoteCounts, PhotographyError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<VoteCounts, PhotographyError, _>(async move |connection| {
                lock_actor(connection, user_id).await?;
                lock_comment(connection, comment_id).await?;
                diesel::insert_into(photograph_comment_votes::table)
                    .values((
                        photograph_comment_votes::photograph_comment_id.eq(comment_id),
                        photograph_comment_votes::user_id.eq(user_id),
                        photograph_comment_votes::is_upvote.eq(is_upvote),
                    ))
                    .on_conflict((
                        photograph_comment_votes::photograph_comment_id,
                        photograph_comment_votes::user_id,
                    ))
                    .do_update()
                    .set(photograph_comment_votes::is_upvote.eq(is_upvote))
                    .execute(&mut *connection)
                    .await?;
                let counts = comment_vote_counts(connection, comment_id).await?;
                diesel::update(
                    photograph_comments::table
                        .filter(photograph_comments::photograph_comment_id.eq(comment_id)),
                )
                .set((
                    photograph_comments::photograph_comment_total_upvotes.eq(counts.upvote_count),
                    photograph_comments::photograph_comment_total_downvotes
                        .eq(counts.downvote_count),
                ))
                .execute(&mut *connection)
                .await?;
                Ok(counts)
            })
            .await
    }

    pub async fn rescind_comment_vote(
        &self,
        user_id: Uuid,
        comment_id: Uuid,
    ) -> Result<VoteCounts, PhotographyError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<VoteCounts, PhotographyError, _>(async move |connection| {
                lock_actor(connection, user_id).await?;
                lock_comment(connection, comment_id).await?;
                let affected = diesel::delete(
                    photograph_comment_votes::table.filter(
                        photograph_comment_votes::photograph_comment_id
                            .eq(comment_id)
                            .and(photograph_comment_votes::user_id.eq(user_id)),
                    ),
                )
                .execute(&mut *connection)
                .await?;
                if affected == 0 {
                    return Err(PhotographyError::VoteNotFound);
                }
                let counts = comment_vote_counts(connection, comment_id).await?;
                diesel::update(
                    photograph_comments::table
                        .filter(photograph_comments::photograph_comment_id.eq(comment_id)),
                )
                .set((
                    photograph_comments::photograph_comment_total_upvotes.eq(counts.upvote_count),
                    photograph_comments::photograph_comment_total_downvotes
                        .eq(counts.downvote_count),
                ))
                .execute(&mut *connection)
                .await?;
                Ok(counts)
            })
            .await
    }
}

async fn lock_photograph(
    connection: &mut AsyncPgConnection,
    photograph_id: Uuid,
) -> Result<(), PhotographyError> {
    photographs::table
        .filter(photographs::photograph_id.eq(photograph_id))
        .select(photographs::photograph_id)
        .for_update()
        .first::<Uuid>(&mut *connection)
        .await
        .optional()?
        .map(|_| ())
        .ok_or(PhotographyError::PhotographNotFound)
}

async fn lock_comment(
    connection: &mut AsyncPgConnection,
    comment_id: Uuid,
) -> Result<(), PhotographyError> {
    photograph_comments::table
        .filter(photograph_comments::photograph_comment_id.eq(comment_id))
        .select(photograph_comments::photograph_comment_id)
        .for_update()
        .first::<Uuid>(&mut *connection)
        .await
        .optional()?
        .map(|_| ())
        .ok_or(PhotographyError::CommentNotFound)
}

async fn lock_actor(
    connection: &mut AsyncPgConnection,
    user_id: Uuid,
) -> Result<(), PhotographyError> {
    lock_active_user(connection, user_id)
        .await
        .map_err(|error| match error {
            ActiveUserWriteError::Inactive => PhotographyError::InactiveAccount,
            ActiveUserWriteError::Denied => PhotographyError::Forbidden,
            ActiveUserWriteError::TargetNotFound => PhotographyError::PhotographNotFound,
            ActiveUserWriteError::Database(error) => PhotographyError::Query(error),
        })
}

async fn photograph_vote_counts(
    connection: &mut AsyncPgConnection,
    photograph_id: Uuid,
) -> Result<VoteCounts, PhotographyError> {
    let rows = photograph_votes::table
        .filter(photograph_votes::photograph_id.eq(photograph_id))
        .group_by(photograph_votes::is_upvote)
        .select((photograph_votes::is_upvote, count_star()))
        .load::<(bool, i64)>(&mut *connection)
        .await?;
    Ok(vote_counts(rows))
}

async fn comment_vote_counts(
    connection: &mut AsyncPgConnection,
    comment_id: Uuid,
) -> Result<VoteCounts, PhotographyError> {
    let rows = photograph_comment_votes::table
        .filter(photograph_comment_votes::photograph_comment_id.eq(comment_id))
        .group_by(photograph_comment_votes::is_upvote)
        .select((photograph_comment_votes::is_upvote, count_star()))
        .load::<(bool, i64)>(&mut *connection)
        .await?;
    Ok(vote_counts(rows))
}

fn vote_counts(rows: Vec<(bool, i64)>) -> VoteCounts {
    let mut counts = VoteCounts {
        upvote_count: 0,
        downvote_count: 0,
    };
    for (is_upvote, count) in rows {
        if is_upvote {
            counts.upvote_count = count;
        } else {
            counts.downvote_count = count;
        }
    }
    counts
}
