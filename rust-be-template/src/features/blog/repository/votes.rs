use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::schema::{comment_votes, comments, post_votes, posts};

use super::super::{domain::vote::VoteCounts, error::BlogError};
use super::{authority::lock_active_user, blog_repository::BlogRepository};

impl BlogRepository {
    pub async fn vote_post(
        &self,
        user_id: Uuid,
        post_id: Uuid,
        is_upvote: bool,
    ) -> Result<VoteCounts, BlogError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<VoteCounts, BlogError, _>(async move |connection| {
                lock_active_user(connection, user_id).await?;
                lock_post_for_vote(connection, post_id).await?;
                diesel::insert_into(post_votes::table)
                    .values((
                        post_votes::post_id.eq(post_id),
                        post_votes::user_id.eq(user_id),
                        post_votes::is_upvote.eq(is_upvote),
                    ))
                    .on_conflict((post_votes::post_id, post_votes::user_id))
                    .do_update()
                    .set(post_votes::is_upvote.eq(is_upvote))
                    .execute(&mut *connection)
                    .await?;
                let counts = post_vote_counts(connection, post_id).await?;
                diesel::update(posts::table.find(post_id))
                    .set((
                        posts::total_upvotes.eq(counts.upvotes),
                        posts::total_downvotes.eq(counts.downvotes),
                    ))
                    .execute(&mut *connection)
                    .await?;
                Ok(counts)
            })
            .await
    }

    pub async fn rescind_post_vote(
        &self,
        user_id: Uuid,
        post_id: Uuid,
    ) -> Result<VoteCounts, BlogError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<VoteCounts, BlogError, _>(async move |connection| {
                lock_active_user(connection, user_id).await?;
                lock_post_for_vote(connection, post_id).await?;
                let deleted = diesel::delete(
                    post_votes::table
                        .filter(post_votes::post_id.eq(post_id))
                        .filter(post_votes::user_id.eq(user_id)),
                )
                .execute(&mut *connection)
                .await?;
                if deleted == 0 {
                    return Err(BlogError::VoteNotFound);
                }
                let counts = post_vote_counts(connection, post_id).await?;
                diesel::update(posts::table.find(post_id))
                    .set((
                        posts::total_upvotes.eq(counts.upvotes),
                        posts::total_downvotes.eq(counts.downvotes),
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
    ) -> Result<VoteCounts, BlogError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<VoteCounts, BlogError, _>(async move |connection| {
                lock_active_user(connection, user_id).await?;
                lock_comment_for_vote(connection, comment_id).await?;
                diesel::insert_into(comment_votes::table)
                    .values((
                        comment_votes::comment_id.eq(comment_id),
                        comment_votes::user_id.eq(user_id),
                        comment_votes::is_upvote.eq(is_upvote),
                    ))
                    .on_conflict((comment_votes::user_id, comment_votes::comment_id))
                    .do_update()
                    .set(comment_votes::is_upvote.eq(is_upvote))
                    .execute(&mut *connection)
                    .await?;
                let counts = comment_vote_counts(connection, comment_id).await?;
                diesel::update(comments::table.find(comment_id))
                    .set((
                        comments::total_upvotes.eq(counts.upvotes),
                        comments::total_downvotes.eq(counts.downvotes),
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
    ) -> Result<VoteCounts, BlogError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<VoteCounts, BlogError, _>(async move |connection| {
                lock_active_user(connection, user_id).await?;
                lock_comment_for_vote(connection, comment_id).await?;
                let deleted = diesel::delete(
                    comment_votes::table
                        .filter(comment_votes::comment_id.eq(comment_id))
                        .filter(comment_votes::user_id.eq(user_id)),
                )
                .execute(&mut *connection)
                .await?;
                if deleted == 0 {
                    return Err(BlogError::VoteNotFound);
                }
                let counts = comment_vote_counts(connection, comment_id).await?;
                diesel::update(comments::table.find(comment_id))
                    .set((
                        comments::total_upvotes.eq(counts.upvotes),
                        comments::total_downvotes.eq(counts.downvotes),
                    ))
                    .execute(&mut *connection)
                    .await?;
                Ok(counts)
            })
            .await
    }
}

async fn lock_post_for_vote(
    connection: &mut diesel_async::AsyncPgConnection,
    post_id: Uuid,
) -> Result<(), BlogError> {
    let locked = posts::table
        .find(post_id)
        .select(posts::post_id)
        .for_update()
        .first::<Uuid>(&mut *connection)
        .await
        .optional()?;
    locked.map(|_| ()).ok_or(BlogError::PostNotFound)
}

async fn lock_comment_for_vote(
    connection: &mut diesel_async::AsyncPgConnection,
    comment_id: Uuid,
) -> Result<(), BlogError> {
    let locked = comments::table
        .find(comment_id)
        .select(comments::comment_id)
        .for_update()
        .first::<Uuid>(&mut *connection)
        .await
        .optional()?;
    locked.map(|_| ()).ok_or(BlogError::CommentNotFound)
}

async fn post_vote_counts(
    connection: &mut diesel_async::AsyncPgConnection,
    post_id: Uuid,
) -> Result<VoteCounts, BlogError> {
    let upvotes = post_votes::table
        .filter(post_votes::post_id.eq(post_id))
        .filter(post_votes::is_upvote.eq(true))
        .count()
        .get_result(&mut *connection)
        .await?;
    let downvotes = post_votes::table
        .filter(post_votes::post_id.eq(post_id))
        .filter(post_votes::is_upvote.eq(false))
        .count()
        .get_result(&mut *connection)
        .await?;
    Ok(VoteCounts { upvotes, downvotes })
}

async fn comment_vote_counts(
    connection: &mut diesel_async::AsyncPgConnection,
    comment_id: Uuid,
) -> Result<VoteCounts, BlogError> {
    let upvotes = comment_votes::table
        .filter(comment_votes::comment_id.eq(comment_id))
        .filter(comment_votes::is_upvote.eq(true))
        .count()
        .get_result(&mut *connection)
        .await?;
    let downvotes = comment_votes::table
        .filter(comment_votes::comment_id.eq(comment_id))
        .filter(comment_votes::is_upvote.eq(false))
        .count()
        .get_result(&mut *connection)
        .await?;
    Ok(VoteCounts { upvotes, downvotes })
}
