use std::{
    collections::VecDeque,
    net::IpAddr,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    },
};

use scc::{Guard, HashMap, HashSet, TreeIndex};
use tokio::sync::{Mutex, broadcast};
use uuid::Uuid;

mod ban;
mod ban_store;
mod event;
mod identity;
mod message;
mod rate;
mod runtime_state;

#[cfg(test)]
mod retained_identity_tests;

pub use crate::features::live_chat::domain::actor::{ChatActor, ChatActorKey};
pub use ban::CachedLiveChatBan;
pub use ban_store::{BanCacheLookup, LIVE_CHAT_BAN_INDEX_MAX_ENTRIES};
pub use event::{ChatConnectionState, LiveChatCacheStats, LiveChatServerEvent, TypingState};
pub use message::{CachedChatMessage, ChatTimelineKey};

use self::{
    message::ChatEvictionKey,
    rate::{LiveChatRateKey, LiveChatRateState},
};

pub const LIVE_CHAT_CACHE_MAX_BYTES: usize = 128 * 1024 * 1024;
pub const LIVE_CHAT_BROADCAST_CAPACITY: usize = 1024;
pub const LIVE_CHAT_ABNORMAL_MESSAGE_LIMIT_PER_SECOND: u32 = 10;
pub const LIVE_CHAT_MAX_CONNECTIONS: u64 = 4_096;
const LIVE_CHAT_TYPING_MAX_ENTRIES: usize = 4_096;
const LIVE_CHAT_RATE_MAX_ENTRIES: usize = 16_384;
const LIVE_CHAT_MESSAGE_FIXED_BYTES: usize = 256;

pub struct LiveChatCache {
    messages_by_id: HashMap<Uuid, Arc<CachedChatMessage>>,
    timeline: TreeIndex<ChatTimelineKey, Uuid>,
    eviction_queue: Mutex<VecDeque<ChatEvictionKey>>,
    typing_by_actor: HashMap<ChatActorKey, TypingState>,
    connected_clients: HashMap<Uuid, ChatConnectionState>,
    disabled_connected_users: HashSet<Uuid>,
    disabled_connected_users_saturated: AtomicBool,
    identity_mutation: Mutex<()>,
    bans_by_user: HashMap<Uuid, CachedLiveChatBan>,
    bans_by_ip: HashMap<IpAddr, CachedLiveChatBan>,
    ban_mutation: Mutex<()>,
    ban_cache_complete: AtomicBool,
    ban_cache_hits: AtomicU64,
    ban_cache_misses: AtomicU64,
    ban_database_read_throughs: AtomicU64,
    ban_rejected_admissions: AtomicU64,
    message_rate_by_key: HashMap<LiveChatRateKey, LiveChatRateState>,
    typing_count: AtomicUsize,
    message_rate_count: AtomicUsize,
    total_bytes: AtomicUsize,
    message_count: AtomicUsize,
    connected_count: AtomicU64,
    max_bytes: usize,
    broadcast_tx: broadcast::Sender<LiveChatServerEvent>,
}

impl LiveChatCache {
    pub fn new(max_bytes: usize) -> Self {
        let (broadcast_tx, _) = broadcast::channel(LIVE_CHAT_BROADCAST_CAPACITY);
        Self {
            messages_by_id: HashMap::new(),
            timeline: TreeIndex::new(),
            eviction_queue: Mutex::new(VecDeque::new()),
            typing_by_actor: HashMap::new(),
            connected_clients: HashMap::new(),
            disabled_connected_users: HashSet::new(),
            disabled_connected_users_saturated: AtomicBool::new(false),
            identity_mutation: Mutex::new(()),
            bans_by_user: HashMap::new(),
            bans_by_ip: HashMap::new(),
            ban_mutation: Mutex::new(()),
            ban_cache_complete: AtomicBool::new(true),
            ban_cache_hits: AtomicU64::new(0),
            ban_cache_misses: AtomicU64::new(0),
            ban_database_read_throughs: AtomicU64::new(0),
            ban_rejected_admissions: AtomicU64::new(0),
            message_rate_by_key: HashMap::new(),
            typing_count: AtomicUsize::new(0),
            message_rate_count: AtomicUsize::new(0),
            total_bytes: AtomicUsize::new(0),
            message_count: AtomicUsize::new(0),
            connected_count: AtomicU64::new(0),
            max_bytes,
            broadcast_tx,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<LiveChatServerEvent> {
        self.broadcast_tx.subscribe()
    }

    /// Clone of the room broadcast sender, used by the SFU to publish
    /// roster/peer-state changes to every connected client.
    pub fn broadcast_sender(&self) -> broadcast::Sender<LiveChatServerEvent> {
        self.broadcast_tx.clone()
    }

    pub fn broadcast(&self, event: LiveChatServerEvent) {
        let _ = self.broadcast_tx.send(event);
    }

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

    async fn evict_over_budget(&self, eviction_queue: &mut VecDeque<ChatEvictionKey>) {
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

    pub async fn stats(&self) -> LiveChatCacheStats {
        let guard = Guard::new();
        let mut iter = self.timeline.iter(&guard);
        let oldest_cached_at = iter.next().and_then(|(_, message_id)| {
            self.messages_by_id
                .read_sync(message_id, |_, message| message.message_created_at)
        });
        let newest_cached_at = match iter.next_back().and_then(|(_, message_id)| {
            self.messages_by_id
                .read_sync(message_id, |_, message| message.message_created_at)
        }) {
            Some(value) => Some(value),
            None => oldest_cached_at,
        };

        LiveChatCacheStats {
            max_bytes: self.max_bytes,
            used_bytes: self.total_bytes.load(Ordering::SeqCst),
            message_count: self.message_count.load(Ordering::SeqCst),
            oldest_cached_at,
            newest_cached_at,
            active_typing_count: self.typing_by_actor.len(),
            connected_count: self.connected_count(),
        }
    }
}

impl Default for LiveChatCache {
    fn default() -> Self {
        Self::new(LIVE_CHAT_CACHE_MAX_BYTES)
    }
}
