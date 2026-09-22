//! Persist moderation before invalidating the cache and notifying connected readers.

use super::{cache::LiveChatServerEvent, live_chat_service::LiveChatService};
use crate::features::live_chat::error::LiveChatError;
use uuid::Uuid;

impl LiveChatService {
    /// Repeated deletion is successful and republishes invalidation for client recovery.
    pub async fn delete_message(&self, actor: Uuid, message: Uuid) -> Result<(), LiveChatError> {
        self.repository.delete_message(actor, message).await?;
        self.cache.remove_message(message).await;
        self.cache.broadcast(LiveChatServerEvent::MessageDeleted {
            live_chat_message_id: message,
        });
        tracing::info!(actor_user_id = %actor, live_chat_message_id = %message, "Live chat message deleted");
        Ok(())
    }
}
