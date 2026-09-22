//! Bounded cache invalidation for moderated messages.

use super::{ChatTimelineKey, LiveChatCache};
use std::sync::atomic::Ordering;
use uuid::Uuid;

impl LiveChatCache {
    /// Remove all indexes and byte accounting under the same lock as cache admission.
    pub async fn remove_message(&self, id: Uuid) {
        let mut queue = self.eviction_queue.lock().await;
        if let Some((_, message)) = self.messages_by_id.remove_async(&id).await {
            let _ = self
                .timeline
                .remove_async(&ChatTimelineKey::from_message(&message))
                .await;
            queue.retain(|entry| entry.live_chat_message_id != id);
            self.total_bytes
                .fetch_sub(message.estimated_bytes(), Ordering::SeqCst);
            self.message_count.fetch_sub(1, Ordering::SeqCst);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::live_chat::{
        api::binary_codec::encode_server_event,
        service::cache::{CachedChatMessage, LiveChatServerEvent},
    };

    #[tokio::test]
    async fn deletion_removes_indexes_and_accounting_idempotently() {
        let cache = LiveChatCache::new(4096);
        let id = Uuid::now_v7();
        cache
            .append_persisted_chat_message(CachedChatMessage {
                live_chat_message_id: id,
                room_key: "main".to_owned(),
                user_id: None,
                guest_ip: None,
                sender_kind: 2,
                sender_display_name: "Guest".to_owned(),
                sender_country_flag: None,
                user_profile_picture_url: None,
                message_body: "moderated content".to_owned(),
                message_created_at: chrono::Utc::now(),
                message_edited_at: None,
                message_deleted_at: None,
            })
            .await;
        assert!(cache.total_bytes.load(Ordering::SeqCst) > 0);
        cache.remove_message(id).await;
        cache.remove_message(id).await;
        assert!(cache.get_recent_chat_messages(50).await.is_empty());
        assert_eq!(cache.total_bytes.load(Ordering::SeqCst), 0);
        assert_eq!(cache.message_count.load(Ordering::SeqCst), 0);
        assert!(cache.eviction_queue.lock().await.is_empty());
    }

    #[test]
    fn deletion_frame_contains_only_opcode_and_message_id() -> Result<(), Box<dyn std::error::Error>>
    {
        let id = Uuid::from_u128(1);
        let event = LiveChatServerEvent::MessageDeleted {
            live_chat_message_id: id,
        };
        let bytes = encode_server_event(&event)?;
        assert_eq!(bytes.len(), 17);
        assert_eq!(bytes[0], 0x88);
        assert_eq!(&bytes[1..], id.as_bytes());
        let json = serde_json::to_value(event)?;
        assert_eq!(json["type"], "message_deleted");
        assert_eq!(json["live_chat_message_id"], id.to_string());
        Ok(())
    }
}
