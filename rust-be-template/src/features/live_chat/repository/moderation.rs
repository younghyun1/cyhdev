//! Transactional moderation with database-current authority.

use chrono::Utc;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use super::live_chat_repository::LiveChatRepository;
use crate::{
    features::live_chat::error::LiveChatError,
    persistence::active_user::{ActiveUserWriteError, lock_active_superuser},
    schema::live_chat_messages,
};

impl LiveChatRepository {
    /// Erase message content, retaining the row as an idempotent pagination tombstone.
    pub async fn delete_message(&self, actor: Uuid, message: Uuid) -> Result<(), LiveChatError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<(), LiveChatError, _>(async move |connection| {
                lock_active_superuser(connection, actor)
                    .await
                    .map_err(|error| match error {
                        ActiveUserWriteError::Inactive => LiveChatError::Unauthorized,
                        ActiveUserWriteError::Denied | ActiveUserWriteError::TargetNotFound => {
                            LiveChatError::Forbidden
                        }
                        ActiveUserWriteError::Database(error) => LiveChatError::Database(error),
                    })?;
                diesel::update(
                    live_chat_messages::table
                        .find(message)
                        .filter(live_chat_messages::message_deleted_at.is_null()),
                )
                .set((
                    live_chat_messages::message_body.eq(""),
                    live_chat_messages::message_deleted_at.eq(Some(Utc::now())),
                ))
                .execute(connection)
                .await?;
                Ok(())
            })
            .await
    }
}
