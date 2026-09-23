use std::{
    collections::VecDeque,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicI64, AtomicU64, AtomicUsize, Ordering},
    },
};

use ipnet::IpNet;
use scc::{Guard, HashMap, HashSet, TreeIndex};
use tokio::sync::{Mutex, broadcast};
use uuid::Uuid;

mod ban;
mod ban_store;
mod broadcast_frame;
mod connections;
mod event;
mod identity;
mod message;
mod moderation;
mod rate;
mod timeline;
mod typing;

#[cfg(test)]
mod retained_identity_tests;

pub use crate::features::live_chat::domain::actor::{ChatActor, ChatActorKey};
pub use ban::CachedLiveChatBan;
pub use ban_store::{BanCacheLookup, LIVE_CHAT_BAN_INDEX_MAX_ENTRIES};
pub use broadcast_frame::LiveChatBroadcast;
pub use connections::{ConnectionAdmission, LIVE_CHAT_MAX_CONNECTIONS_PER_ADDRESS};
pub use event::{ChatConnectionState, LiveChatCacheStats, LiveChatServerEvent, TypingState};
pub use message::{CachedChatMessage, ChatTimelineKey};
pub use rate::MessageRateDecision;

use self::{
    message::ChatEvictionKey,
    rate::{LiveChatRateKey, LiveChatRateState},
};
use crate::features::live_chat::domain::ip_prefix::LiveChatIpPrefix;

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
    /// Set while a coalesced typing broadcast is scheduled.
    typing_broadcast_pending: AtomicBool,
    /// Unix milliseconds of the last typing broadcast.
    typing_broadcast_at_millis: AtomicI64,
    connected_clients: HashMap<Uuid, ChatConnectionState>,
    /// Live connections per address group; each entry needs at least one
    /// connection, so the map is bounded by `LIVE_CHAT_MAX_CONNECTIONS`.
    connections_by_prefix: HashMap<LiveChatIpPrefix, usize>,
    disabled_connected_users: HashSet<Uuid>,
    disabled_connected_users_saturated: AtomicBool,
    identity_mutation: Mutex<()>,
    bans_by_user: HashMap<Uuid, CachedLiveChatBan>,
    /// Keyed by banned network: /32 or /128 hosts and IPv6 /64 subscriber groups.
    bans_by_ip: HashMap<IpNet, CachedLiveChatBan>,
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
    broadcast_tx: broadcast::Sender<Arc<LiveChatBroadcast>>,
}

impl LiveChatCache {
    pub fn new(max_bytes: usize) -> Self {
        let (broadcast_tx, _) = broadcast::channel(LIVE_CHAT_BROADCAST_CAPACITY);
        Self {
            messages_by_id: HashMap::new(),
            timeline: TreeIndex::new(),
            eviction_queue: Mutex::new(VecDeque::new()),
            typing_by_actor: HashMap::new(),
            typing_broadcast_pending: AtomicBool::new(false),
            typing_broadcast_at_millis: AtomicI64::new(0),
            connected_clients: HashMap::new(),
            connections_by_prefix: HashMap::new(),
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

    pub fn subscribe(&self) -> broadcast::Receiver<Arc<LiveChatBroadcast>> {
        self.broadcast_tx.subscribe()
    }

    /// Clone of the room broadcast sender, used by the SFU to publish
    /// roster/peer-state changes to every connected client.
    pub fn broadcast_sender(&self) -> broadcast::Sender<Arc<LiveChatBroadcast>> {
        self.broadcast_tx.clone()
    }

    pub fn broadcast(&self, event: LiveChatServerEvent) {
        let _ = self.broadcast_tx.send(LiveChatBroadcast::new(event));
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
