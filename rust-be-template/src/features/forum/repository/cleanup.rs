//! Bounded expiry cleanup for the durable notification inbox.

use chrono::{DateTime, Utc};
use diesel::{ExpressionMethods, QueryDsl, dsl::exists};
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{features::forum::{domain::models::ForumNotificationPruneReport, error::ForumError, repository::forum_repository::ForumRepository}, schema::forum_notifications};

pub const FORUM_NOTIFICATION_CLEANUP_BATCH: i64 = 512;

impl ForumRepository {
    pub async fn prune_expired_notifications(&self, now: DateTime<Utc>) -> Result<ForumNotificationPruneReport, ForumError> {
        let mut connection = self.connection().await?;
        connection.transaction::<ForumNotificationPruneReport, ForumError, _>(async move |connection| {
            let ids = forum_notifications::table
                .filter(forum_notifications::forum_notification_expires_at.le(now))
                .order((forum_notifications::forum_notification_expires_at.asc(), forum_notifications::forum_notification_id.asc()))
                .select(forum_notifications::forum_notification_id)
                .limit(FORUM_NOTIFICATION_CLEANUP_BATCH).for_update().skip_locked()
                .load::<Uuid>(&mut *connection).await?;
            let deleted = if ids.is_empty() { 0 } else {
                diesel::delete(forum_notifications::table.filter(forum_notifications::forum_notification_id.eq_any(&ids)))
                    .execute(&mut *connection).await?
            };
            let remaining_expired = diesel::select(exists(forum_notifications::table
                .filter(forum_notifications::forum_notification_expires_at.le(now))))
                .get_result::<bool>(&mut *connection).await?;
            Ok(ForumNotificationPruneReport { deleted, remaining_expired })
        }).await
    }
}
