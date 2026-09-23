//! Post and comment votes with denormalized totals recomputed under row locks.

use chrono::{DateTime, Utc};
use diesel::{
    AggregateExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, dsl::count,
};
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use crate::schema::{comment_votes, comments, post_votes, posts};

use super::super::{domain::vote::VoteCounts, error::BlogError};
use super::{
    authority::{lock_active_user, require_visible_post},
    blog_repository::BlogRepository,
};

/// Whether a vote is being cast (subject to draft visibility) or withdrawn.
#[derive(Clone, Copy, PartialEq, Eq)]
enum VoteAction {
    Cast,
    Rescind,
}

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
                lock_post_for_vote(connection, user_id, post_id, VoteAction::Cast).await?;
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
                store_post_counts(connection, post_id).await
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
                lock_post_for_vote(connection, user_id, post_id, VoteAction::Rescind).await?;
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
                store_post_counts(connection, post_id).await
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
                lock_comment_for_vote(connection, user_id, comment_id, VoteAction::Cast).await?;
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
                store_comment_counts(connection, comment_id).await
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
                lock_comment_for_vote(connection, user_id, comment_id, VoteAction::Rescind).await?;
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
                store_comment_counts(connection, comment_id).await
            })
            .await
    }
}

/// Locks the post row that serializes its vote recount. Casting a vote on a
/// draft is refused for non-managers; withdrawing an earlier vote is allowed.
async fn lock_post_for_vote(
    connection: &mut AsyncPgConnection,
    user_id: Uuid,
    post_id: Uuid,
    action: VoteAction,
) -> Result<(), BlogError> {
    let published = posts::table
        .find(post_id)
        .select(posts::post_is_published)
        .for_update()
        .first::<bool>(&mut *connection)
        .await
        .optional()?
        .ok_or(BlogError::PostNotFound)?;
    if action == VoteAction::Cast {
        require_visible_post(connection, user_id, published).await?;
    }
    Ok(())
}

/// Locks the comment row that serializes its vote recount. New votes are
/// refused on tombstones and on comments of drafts the actor cannot manage.
async fn lock_comment_for_vote(
    connection: &mut AsyncPgConnection,
    user_id: Uuid,
    comment_id: Uuid,
    action: VoteAction,
) -> Result<(), BlogError> {
    let (post_id, deleted_at) = comments::table
        .find(comment_id)
        .select((comments::post_id, comments::comment_deleted_at))
        .for_update()
        .first::<(Uuid, Option<DateTime<Utc>>)>(&mut *connection)
        .await
        .optional()?
        .ok_or(BlogError::CommentNotFound)?;
    if action == VoteAction::Rescind {
        return Ok(());
    }
    if deleted_at.is_some() {
        return Err(BlogError::CommentNotFound);
    }
    let published = posts::table
        .find(post_id)
        .select(posts::post_is_published)
        .first::<bool>(&mut *connection)
        .await?;
    require_visible_post(connection, user_id, published).await
}

/// Recounts one post's votes with a single filtered aggregate and stores it.
async fn store_post_counts(
    connection: &mut AsyncPgConnection,
    post_id: Uuid,
) -> Result<VoteCounts, BlogError> {
    let (upvotes, downvotes) = post_votes::table
        .filter(post_votes::post_id.eq(post_id))
        .select((
            count(post_votes::vote_id).aggregate_filter(post_votes::is_upvote.eq(true)),
            count(post_votes::vote_id).aggregate_filter(post_votes::is_upvote.eq(false)),
        ))
        .first::<(i64, i64)>(&mut *connection)
        .await?;
    diesel::update(posts::table.find(post_id))
        .set((
            posts::total_upvotes.eq(upvotes),
            posts::total_downvotes.eq(downvotes),
        ))
        .execute(&mut *connection)
        .await?;
    Ok(VoteCounts { upvotes, downvotes })
}

/// Recounts one comment's votes with a single filtered aggregate and stores it.
async fn store_comment_counts(
    connection: &mut AsyncPgConnection,
    comment_id: Uuid,
) -> Result<VoteCounts, BlogError> {
    let (upvotes, downvotes) = comment_votes::table
        .filter(comment_votes::comment_id.eq(comment_id))
        .select((
            count(comment_votes::vote_id).aggregate_filter(comment_votes::is_upvote.eq(true)),
            count(comment_votes::vote_id).aggregate_filter(comment_votes::is_upvote.eq(false)),
        ))
        .first::<(i64, i64)>(&mut *connection)
        .await?;
    diesel::update(comments::table.find(comment_id))
        .set((
            comments::total_upvotes.eq(upvotes),
            comments::total_downvotes.eq(downvotes),
        ))
        .execute(&mut *connection)
        .await?;
    Ok(VoteCounts { upvotes, downvotes })
}
