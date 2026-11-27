use diesel::{
    BoolExpressionMethods, ExpressionMethods, NullableExpressionMethods, OptionalExtension,
    QueryDsl, dsl::count_star,
};
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use jiff::Timestamp;
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
                store_photograph_counts(connection, photograph_id).await
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
                store_photograph_counts(connection, photograph_id).await
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
                if lock_comment(connection, comment_id).await?.is_some() {
                    // Tombstones keep their totals but accept no new votes.
                    return Err(PhotographyError::CommentNotFound);
                }
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
                store_comment_counts(connection, comment_id).await
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
                store_comment_counts(connection, comment_id).await
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

/// Locks the comment row that serializes its vote recount and returns its
/// deletion time.
async fn lock_comment(
    connection: &mut AsyncPgConnection,
    comment_id: Uuid,
) -> Result<Option<Timestamp>, PhotographyError> {
    photograph_comments::table
        .filter(photograph_comments::photograph_comment_id.eq(comment_id))
        .select(photograph_comments::photograph_comment_deleted_at)
        .for_update()
        .first::<jiff_diesel::NullableTimestamp>(&mut *connection)
        .await
        .optional()?
        .map(jiff_diesel::NullableTimestamp::to_jiff)
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

/// Recounts one photograph's votes and stores the totals in one statement.
///
/// The caller holds the photograph row lock, so the row exists and the
/// returned totals are the ones it now holds. A scalar `count(*)` subquery
/// always yields a row, which is why its nullable type is asserted away.
async fn store_photograph_counts(
    connection: &mut AsyncPgConnection,
    photograph_id: Uuid,
) -> Result<VoteCounts, PhotographyError> {
    let votes = |is_upvote: bool| {
        photograph_votes::table
            .filter(photograph_votes::photograph_id.eq(photograph_id))
            .filter(photograph_votes::is_upvote.eq(is_upvote))
            .select(count_star())
            .single_value()
            .assume_not_null()
    };
    let (upvote_count, downvote_count) =
        diesel::update(photographs::table.filter(photographs::photograph_id.eq(photograph_id)))
            .set((
                photographs::photograph_total_upvotes.eq(votes(true)),
                photographs::photograph_total_downvotes.eq(votes(false)),
            ))
            .returning((
                photographs::photograph_total_upvotes,
                photographs::photograph_total_downvotes,
            ))
            .get_result::<(i64, i64)>(&mut *connection)
            .await?;
    Ok(VoteCounts {
        upvote_count,
        downvote_count,
    })
}

/// Recounts one comment's votes and stores the totals in one statement, under
/// the comment row lock its caller holds.
async fn store_comment_counts(
    connection: &mut AsyncPgConnection,
    comment_id: Uuid,
) -> Result<VoteCounts, PhotographyError> {
    let votes = |is_upvote: bool| {
        photograph_comment_votes::table
            .filter(photograph_comment_votes::photograph_comment_id.eq(comment_id))
            .filter(photograph_comment_votes::is_upvote.eq(is_upvote))
            .select(count_star())
            .single_value()
            .assume_not_null()
    };
    let (upvote_count, downvote_count) = diesel::update(
        photograph_comments::table
            .filter(photograph_comments::photograph_comment_id.eq(comment_id)),
    )
    .set((
        photograph_comments::photograph_comment_total_upvotes.eq(votes(true)),
        photograph_comments::photograph_comment_total_downvotes.eq(votes(false)),
    ))
    .returning((
        photograph_comments::photograph_comment_total_upvotes,
        photograph_comments::photograph_comment_total_downvotes,
    ))
    .get_result::<(i64, i64)>(&mut *connection)
    .await?;
    Ok(VoteCounts {
        upvote_count,
        downvote_count,
    })
}
