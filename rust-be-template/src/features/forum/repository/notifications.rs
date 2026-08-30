//! Recipient-scoped notification inbox reads and idempotent read markers.

use chrono::{DateTime, Utc};
use diesel::{BoolExpressionMethods, ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl};
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    features::{
        accounts::domain::public_author::PublicAuthor,
        forum::{
            domain::models::{ForumNotificationPage, ForumNotificationView, ForumTimestampCursor},
            domain::validation::ForumPageSize,
            error::ForumError,
            repository::{
                forum_repository::ForumRepository,
                records::{ForumNotificationRow, notification_from_row},
            },
        },
    },
    persistence::public_authors::load_public_authors,
    schema::{forum_notifications, forum_topics},
};

impl ForumRepository {
    pub async fn notification_page(
        &self,
        recipient_user_id: Uuid,
        before: Option<ForumTimestampCursor>,
        page_size: ForumPageSize,
        now: DateTime<Utc>,
    ) -> Result<ForumNotificationPage, ForumError> {
        let mut connection = self.connection().await?;
        let mut query = forum_notifications::table
            .inner_join(forum_topics::table.on(
                forum_topics::forum_topic_id.eq(forum_notifications::forum_notification_topic_id),
            ))
            .filter(forum_notifications::forum_notification_recipient_user_id.eq(recipient_user_id))
            .filter(forum_notifications::forum_notification_expires_at.gt(now))
            .select((
                forum_notifications::forum_notification_id,
                forum_notifications::forum_notification_recipient_user_id,
                forum_notifications::forum_notification_actor_user_id,
                forum_notifications::forum_notification_topic_id,
                forum_notifications::forum_notification_reply_id,
                forum_notifications::forum_notification_kind,
                forum_notifications::forum_notification_created_at,
                forum_notifications::forum_notification_expires_at,
                forum_notifications::forum_notification_read_at,
                forum_topics::forum_topic_title,
                forum_topics::forum_topic_content_state,
            ))
            .order((
                forum_notifications::forum_notification_created_at.desc(),
                forum_notifications::forum_notification_id.desc(),
            ))
            .into_boxed();
        if let Some(cursor) = before {
            query = query.filter(
                forum_notifications::forum_notification_created_at
                    .lt(cursor.created_at)
                    .or(forum_notifications::forum_notification_created_at
                        .eq(cursor.created_at)
                        .and(forum_notifications::forum_notification_id.lt(cursor.item_id))),
            );
        }
        let mut rows = query
            .limit(i64::from(page_size.into_inner()) + 1)
            .load::<ForumNotificationRow>(&mut connection)
            .await?;
        let next_cursor = if rows.len() > usize::from(page_size.into_inner()) {
            rows.pop();
            rows.last().map(|row| ForumTimestampCursor {
                created_at: row.6,
                item_id: row.0,
            })
        } else {
            None
        };
        let notifications = rows
            .into_iter()
            .map(notification_from_row)
            .collect::<Vec<_>>();
        let actor_ids = notifications
            .iter()
            .map(|notification| notification.actor_user_id)
            .collect::<Vec<_>>();
        let authors = load_public_authors(&mut connection, &actor_ids).await?;
        let items = notifications
            .into_iter()
            .map(|notification| {
                let actor = authors
                    .get(&notification.actor_user_id)
                    .cloned()
                    .unwrap_or_else(PublicAuthor::deleted);
                ForumNotificationView {
                    notification,
                    actor,
                }
            })
            .collect();
        Ok(ForumNotificationPage { items, next_cursor })
    }

    pub async fn mark_notification_read(
        &self,
        recipient_user_id: Uuid,
        notification_id: Uuid,
        now: DateTime<Utc>,
    ) -> Result<DateTime<Utc>, ForumError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<DateTime<Utc>, ForumError, _>(async move |connection| {
                let row = forum_notifications::table
                    .filter(forum_notifications::forum_notification_id.eq(notification_id))
                    .filter(
                        forum_notifications::forum_notification_recipient_user_id
                            .eq(recipient_user_id),
                    )
                    .select((
                        forum_notifications::forum_notification_expires_at,
                        forum_notifications::forum_notification_read_at,
                    ))
                    .for_update()
                    .first::<(DateTime<Utc>, Option<DateTime<Utc>>)>(&mut *connection)
                    .await
                    .optional()?
                    .ok_or(ForumError::NotificationNotFound)?;
                if row.0 <= now {
                    return Err(ForumError::NotificationNotFound);
                }
                if let Some(read_at) = row.1 {
                    return Ok(read_at);
                }
                diesel::update(
                    forum_notifications::table
                        .filter(forum_notifications::forum_notification_id.eq(notification_id)),
                )
                .set(forum_notifications::forum_notification_read_at.eq(Some(now)))
                .execute(&mut *connection)
                .await?;
                Ok(now)
            })
            .await
    }
}
