//! Byte-bounded message timeline: admission, FIFO eviction, and range reads.

use std::{
    collections::VecDeque,
    sync::{Arc, atomic::Ordering},
};

use scc::Guard;
use uuid::Uuid;

use super::{CachedChatMessage, ChatTimelineKey, LiveChatCache, message::ChatEvictionKey};

impl LiveChatCache {
    pub async fn clear_messages(&self) {
        let mut eviction_queue = self.eviction_queue.lock().await;
        self.messages_by_id.clear_async().await;
        self.timeline.clear();
        eviction_queue.clear();
        self.total_bytes.store(0, Ordering::SeqCst);
        self.message_count.store(0, Ordering::SeqCst);
    }

    pub async fn append_persisted_chat_message(&self, message: CachedChatMessage) {
        let mut eviction_queue = self.eviction_queue.lock().await;
        let estimated_bytes = message.estimated_bytes();
        let timeline_key = ChatTimelineKey::from_message(&message);
        let message_id = message.live_chat_message_id;
        let message = Arc::new(message);

        if let Some(previous) = self.messages_by_id.upsert_async(message_id, message).await {
            let previous_timeline_key = ChatTimelineKey::from_message(&previous);
            let _ = self.timeline.remove_async(&previous_timeline_key).await;
            eviction_queue.retain(|entry| entry.live_chat_message_id != message_id);
            self.total_bytes
                .fetch_sub(previous.estimated_bytes(), Ordering::SeqCst);
            self.message_count.fetch_sub(1, Ordering::SeqCst);
        }

        let _ = self
            .timeline
            .insert_async(timeline_key.clone(), message_id)
            .await;
        eviction_queue.push_back(ChatEvictionKey {
            live_chat_message_id: message_id,
            timeline_key,
            estimated_bytes,
        });
        self.total_bytes
            .fetch_add(estimated_bytes, Ordering::SeqCst);
        self.message_count.fetch_add(1, Ordering::SeqCst);
        self.evict_over_budget(&mut eviction_queue).await;
    }

    pub(super) async fn evict_over_budget(&self, eviction_queue: &mut VecDeque<ChatEvictionKey>) {
        while self.total_bytes.load(Ordering::SeqCst) > self.max_bytes {
            let eviction_key = match eviction_queue.pop_front() {
                Some(entry) => entry,
                None => return,
            };
            if let Some((_, removed)) = self
                .messages_by_id
                .remove_if_async(&eviction_key.live_chat_message_id, |message| {
                    ChatTimelineKey::from_message(message) == eviction_key.timeline_key
                        && message.estimated_bytes() == eviction_key.estimated_bytes
                })
                .await
            {
                let _ = self.timeline.remove_async(&eviction_key.timeline_key).await;
                self.total_bytes
                    .fetch_sub(removed.estimated_bytes(), Ordering::SeqCst);
                self.message_count.fetch_sub(1, Ordering::SeqCst);
            }
        }
    }

    pub async fn get_recent_chat_messages(&self, limit: usize) -> Vec<CachedChatMessage> {
        let ids = {
            let guard = Guard::new();
            let mut ids = Vec::with_capacity(limit);
            let mut iter = self.timeline.iter(&guard);
            while ids.len() < limit {
                match iter.next_back() {
                    Some((_, message_id)) => ids.push(*message_id),
                    None => break,
                }
            }
            ids.reverse();
            ids
        };
        self.messages_for_ids(ids).await
    }

    pub async fn get_chat_messages_before(
        &self,
        before: ChatTimelineKey,
        limit: usize,
    ) -> Vec<CachedChatMessage> {
        let ids = {
            let guard = Guard::new();
            let mut ids = Vec::with_capacity(limit);
            let mut range = self.timeline.range(..before, &guard);
            while ids.len() < limit {
                match range.next_back() {
                    Some((_, message_id)) => ids.push(*message_id),
                    None => break,
                }
            }
            ids.reverse();
            ids
        };
        self.messages_for_ids(ids).await
    }

    async fn messages_for_ids(&self, ids: Vec<Uuid>) -> Vec<CachedChatMessage> {
        let mut messages = Vec::with_capacity(ids.len());
        for message_id in ids {
            if let Some(message) = self
                .messages_by_id
                .read_async(&message_id, |_, message| (**message).clone())
                .await
            {
                messages.push(message);
            }
        }
        messages
    }

    pub async fn get_timeline_key_for_message(&self, message_id: Uuid) -> Option<ChatTimelineKey> {
        self.messages_by_id
            .read_async(&message_id, |_, message| {
                ChatTimelineKey::from_message(message)
            })
            .await
    }
}
