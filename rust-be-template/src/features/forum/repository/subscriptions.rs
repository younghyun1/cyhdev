//! Fixed-capacity durable topic subscriptions.

use chrono::{DateTime, Utc};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, dsl::count_star};
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    features::forum::{
        error::ForumError,
        repository::{
            enums::DbForumContentState, forum_repository::ForumRepository,
            records::NewForumSubscriptionRecord,
        },
    },
    schema::{forum_topic_subscriptions, forum_topics},
};

pub const MAX_FORUM_TOPIC_SUBSCRIPTIONS: i64 = 4_096;

impl ForumRepository {
    pub async fn subscribe(
        &self,
        user_id: Uuid,
        topic_id: Uuid,
        now: DateTime<Utc>,
    ) -> Result<bool, ForumError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<bool, ForumError, _>(async move |connection| {
                lock_topic(connection, topic_id).await?;
                ensure_subscription(connection, user_id, topic_id, now).await
            })
            .await
    }

    pub async fn unsubscribe(&self, user_id: Uuid, topic_id: Uuid) -> Result<bool, ForumError> {
        let mut connection = self.connection().await?;
        let deleted = diesel::delete(
            forum_topic_subscriptions::table
                .filter(forum_topic_subscriptions::forum_topic_subscription_topic_id.eq(topic_id))
                .filter(forum_topic_subscriptions::forum_topic_subscription_user_id.eq(user_id)),
        )
        .execute(&mut connection)
        .await?;
        Ok(deleted > 0)
    }
}

pub(super) async fn ensure_subscription(
    connection: &mut AsyncPgConnection,
    user_id: Uuid,
    topic_id: Uuid,
    now: DateTime<Utc>,
) -> Result<bool, ForumError> {
    let existing = forum_topic_subscriptions::table
        .filter(forum_topic_subscriptions::forum_topic_subscription_topic_id.eq(topic_id))
        .filter(forum_topic_subscriptions::forum_topic_subscription_user_id.eq(user_id))
        .select(forum_topic_subscriptions::forum_topic_subscription_id)
        .first::<Uuid>(&mut *connection)
        .await
        .optional()?;
    if existing.is_some() {
        return Ok(false);
    }
    let count = forum_topic_subscriptions::table
        .filter(forum_topic_subscriptions::forum_topic_subscription_topic_id.eq(topic_id))
        .select(count_star())
        .first::<i64>(&mut *connection)
        .await?;
    if count >= MAX_FORUM_TOPIC_SUBSCRIPTIONS {
        return Err(ForumError::SubscriptionSaturated {
            maximum: MAX_FORUM_TOPIC_SUBSCRIPTIONS,
        });
    }
    diesel::insert_into(forum_topic_subscriptions::table)
        .values(NewForumSubscriptionRecord {
            forum_topic_subscription_topic_id: topic_id,
            forum_topic_subscription_user_id: user_id,
            forum_topic_subscription_created_at: now,
        })
        .execute(&mut *connection)
        .await?;
    Ok(true)
}

async fn lock_topic(connection: &mut AsyncPgConnection, topic_id: Uuid) -> Result<(), ForumError> {
    let topic = forum_topics::table
        .filter(forum_topics::forum_topic_id.eq(topic_id))
        .select(forum_topics::forum_topic_content_state)
        .for_update()
        .first::<DbForumContentState>(&mut *connection)
        .await
        .optional()?;
    match topic {
        Some(DbForumContentState::Visible | DbForumContentState::Hidden) => Ok(()),
        Some(DbForumContentState::Deleted) => Err(ForumError::ContentStateConflict),
        None => Err(ForumError::TopicNotFound),
    }
}
