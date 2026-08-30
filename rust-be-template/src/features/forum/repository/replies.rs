//! Transactional flat replies, counters, subscriptions, and notification fanout.

use chrono::{DateTime, Days, Utc};
use diesel::{ExpressionMethods, IntoSql, OptionalExtension, QueryDsl, SelectableHelper};
use diesel::sql_types::{Timestamptz, Uuid as SqlUuid};
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    features::forum::{
        domain::{models::ForumMutationReceipt, validation::ForumBody},
        error::ForumError,
        repository::{
            enums::{DbForumContentState, DbForumNotificationKind, DbForumTopicAccessState},
            forum_repository::ForumRepository,
            records::{ForumReplyRecord, NewForumReplyRecord},
            subscriptions::ensure_subscription,
        },
    },
    schema::{forum_notifications, forum_replies, forum_topic_subscriptions, forum_topics, sql_types::ForumNotificationKind as ForumNotificationKindSql},
};

const DELETED_CONTENT_BODY: &str = "[deleted]";

impl ForumRepository {
    pub async fn create_reply(
        &self,
        actor_user_id: Uuid,
        topic_id: Uuid,
        body: &ForumBody,
        now: DateTime<Utc>,
    ) -> Result<ForumMutationReceipt, ForumError> {
        let expires_at = now.checked_add_days(Days::new(90)).ok_or(ForumError::CountOverflow)?;
        let mut connection = self.connection().await?;
        connection.transaction::<ForumMutationReceipt, ForumError, _>(async move |connection| {
            let topic_state = forum_topics::table
                .filter(forum_topics::forum_topic_id.eq(topic_id))
                .select((forum_topics::forum_topic_content_state, forum_topics::forum_topic_access_state,
                    forum_topics::forum_topic_updated_at, forum_topics::forum_topic_last_activity_at))
                .for_update().first::<(DbForumContentState, DbForumTopicAccessState, DateTime<Utc>, DateTime<Utc>)>(&mut *connection)
                .await.optional()?.ok_or(ForumError::TopicNotFound)?;
            if topic_state.0 != DbForumContentState::Visible { return Err(ForumError::ContentStateConflict); }
            if topic_state.1 != DbForumTopicAccessState::Open { return Err(ForumError::TopicLocked); }
            match ensure_subscription(connection, actor_user_id, topic_id, now).await {
                Ok(_) | Err(ForumError::SubscriptionSaturated { .. }) => {}
                Err(error) => return Err(error),
            }

            let (reply_id, revision, updated_at) = diesel::insert_into(forum_replies::table)
                .values(NewForumReplyRecord { forum_reply_topic_id: topic_id, forum_reply_author_user_id: actor_user_id,
                    forum_reply_body: body.as_ref(), forum_reply_created_at: now, forum_reply_updated_at: now })
                .returning((forum_replies::forum_reply_id, forum_replies::forum_reply_revision, forum_replies::forum_reply_updated_at))
                .get_result::<(Uuid, i32, DateTime<Utc>)>(&mut *connection).await?;
            diesel::update(forum_topics::table.filter(forum_topics::forum_topic_id.eq(topic_id)))
                .set((forum_topics::forum_topic_reply_count.eq(forum_topics::forum_topic_reply_count + 1),
                    forum_topics::forum_topic_revision.eq(forum_topics::forum_topic_revision + 1),
                    forum_topics::forum_topic_updated_at.eq(topic_state.2.max(now)),
                    forum_topics::forum_topic_last_activity_at.eq(topic_state.3.max(now))))
                .execute(&mut *connection).await?;

            let recipients = forum_topic_subscriptions::table
                .filter(forum_topic_subscriptions::forum_topic_subscription_topic_id.eq(topic_id))
                .filter(forum_topic_subscriptions::forum_topic_subscription_user_id.ne(actor_user_id))
                .select((
                    forum_topic_subscriptions::forum_topic_subscription_user_id,
                    actor_user_id.into_sql::<SqlUuid>(),
                    topic_id.into_sql::<SqlUuid>(),
                    reply_id.into_sql::<SqlUuid>(),
                    DbForumNotificationKind::TopicReply.into_sql::<ForumNotificationKindSql>(),
                    now.into_sql::<Timestamptz>(),
                    expires_at.into_sql::<Timestamptz>(),
                ));
            diesel::insert_into(forum_notifications::table)
                .values(recipients)
                .into_columns((
                    forum_notifications::forum_notification_recipient_user_id,
                    forum_notifications::forum_notification_actor_user_id,
                    forum_notifications::forum_notification_topic_id,
                    forum_notifications::forum_notification_reply_id,
                    forum_notifications::forum_notification_kind,
                    forum_notifications::forum_notification_created_at,
                    forum_notifications::forum_notification_expires_at,
                ))
                .on_conflict((forum_notifications::forum_notification_recipient_user_id, forum_notifications::forum_notification_reply_id))
                .do_nothing().execute(&mut *connection).await?;
            Ok(ForumMutationReceipt { item_id: reply_id, revision, updated_at })
        }).await
    }

    pub async fn update_reply(
        &self,
        actor_user_id: Uuid,
        reply_id: Uuid,
        body: &ForumBody,
        expected_revision: i32,
        now: DateTime<Utc>,
    ) -> Result<ForumMutationReceipt, ForumError> {
        let mut connection = self.connection().await?;
        connection.transaction::<ForumMutationReceipt, ForumError, _>(async move |connection| {
            let record = lock_reply(connection, reply_id).await?;
            validate_author_reply(&record, actor_user_id, expected_revision)?;
            if record.forum_reply_content_state != DbForumContentState::Visible { return Err(ForumError::ContentStateConflict); }
            let (revision, updated_at) = diesel::update(forum_replies::table.filter(forum_replies::forum_reply_id.eq(reply_id)))
                .set((forum_replies::forum_reply_body.eq(body.as_ref()), forum_replies::forum_reply_revision.eq(forum_replies::forum_reply_revision + 1),
                    forum_replies::forum_reply_updated_at.eq(now), forum_replies::forum_reply_edited_at.eq(now)))
                .returning((forum_replies::forum_reply_revision, forum_replies::forum_reply_updated_at))
                .get_result::<(i32, DateTime<Utc>)>(&mut *connection).await?;
            Ok(ForumMutationReceipt { item_id: reply_id, revision, updated_at })
        }).await
    }

    pub async fn delete_reply(
        &self,
        actor_user_id: Uuid,
        reply_id: Uuid,
        expected_revision: i32,
        now: DateTime<Utc>,
    ) -> Result<ForumMutationReceipt, ForumError> {
        let mut connection = self.connection().await?;
        connection.transaction::<ForumMutationReceipt, ForumError, _>(async move |connection| {
            let record = lock_reply(connection, reply_id).await?;
            validate_author_reply(&record, actor_user_id, expected_revision)?;
            if record.forum_reply_content_state == DbForumContentState::Deleted { return Err(ForumError::ContentStateConflict); }
            let (revision, updated_at) = diesel::update(forum_replies::table.filter(forum_replies::forum_reply_id.eq(reply_id)))
                .set((forum_replies::forum_reply_body.eq(DELETED_CONTENT_BODY), forum_replies::forum_reply_content_state.eq(DbForumContentState::Deleted),
                    forum_replies::forum_reply_hidden_at.eq(Option::<DateTime<Utc>>::None), forum_replies::forum_reply_deleted_at.eq(Some(now)),
                    forum_replies::forum_reply_revision.eq(forum_replies::forum_reply_revision + 1), forum_replies::forum_reply_updated_at.eq(now)))
                .returning((forum_replies::forum_reply_revision, forum_replies::forum_reply_updated_at))
                .get_result::<(i32, DateTime<Utc>)>(&mut *connection).await?;
            Ok(ForumMutationReceipt { item_id: reply_id, revision, updated_at })
        }).await
    }
}

pub(super) async fn lock_reply(connection: &mut diesel_async::AsyncPgConnection, reply_id: Uuid) -> Result<ForumReplyRecord, ForumError> {
    forum_replies::table.filter(forum_replies::forum_reply_id.eq(reply_id)).select(ForumReplyRecord::as_select())
        .for_update().first::<ForumReplyRecord>(&mut *connection).await.optional()?.ok_or(ForumError::ReplyNotFound)
}

fn validate_author_reply(record: &ForumReplyRecord, actor_user_id: Uuid, expected_revision: i32) -> Result<(), ForumError> {
    if record.forum_reply_author_user_id != actor_user_id { return Err(ForumError::NotOwner); }
    if record.forum_reply_revision != expected_revision { return Err(ForumError::RevisionConflict); }
    Ok(())
}
