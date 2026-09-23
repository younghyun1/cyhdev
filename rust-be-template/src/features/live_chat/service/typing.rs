//! Coalesced typing broadcasts.
//!
//! Every typing event used to broadcast the whole typing list to every client,
//! so one client could multiply its own event rate by the connection count.
//! Now a broadcast is scheduled only when the visible set changes, or when a
//! refresh arrives after the rebroadcast interval so clients' expiry timers do
//! not clear someone still typing; bursts collapse into one broadcast.

use std::{sync::Arc, time::Duration};

use chrono::{Duration as ChronoDuration, Utc};

use super::{
    super::domain::{actor::ChatActor, message::DEFAULT_LIVE_CHAT_ROOM},
    cache::{ChatActorKey, LiveChatServerEvent, TypingState},
    live_chat_service::LiveChatService,
};

/// Lifetime of a typing entry without a refresh; browsers refresh every 2 s.
pub const LIVE_CHAT_TYPING_TTL: ChronoDuration = ChronoDuration::seconds(4);
/// Window that collapses bursts of typing changes into one broadcast.
const TYPING_BROADCAST_COALESCE: Duration = Duration::from_millis(250);
/// Refreshes rebroadcast at most this often, well inside the TTL browsers use
/// to expire the displayed set.
const TYPING_REBROADCAST_INTERVAL_MILLIS: i64 = 2_000;
/// Typing actors carried by one broadcast.
pub const LIVE_CHAT_TYPING_BROADCAST_MAX_ACTORS: usize = 16;

impl LiveChatService {
    /// Record a start or stop from one connection's actor.
    pub async fn record_typing(self: &Arc<Self>, actor: &ChatActor, is_typing: bool) {
        let now = Utc::now();
        let changed = if is_typing {
            self.cache
                .set_typing(TypingState {
                    actor: actor.clone(),
                    room_key: DEFAULT_LIVE_CHAT_ROOM.to_owned(),
                    expires_at: now + LIVE_CHAT_TYPING_TTL,
                })
                .await
        } else {
            self.cache.clear_typing(&actor.actor_key).await
        };
        let refresh_due = is_typing
            && self
                .cache
                .typing_broadcast_older_than(now, TYPING_REBROADCAST_INTERVAL_MILLIS);
        if changed || refresh_due {
            self.schedule_typing_broadcast();
        }
    }

    /// Clear an actor's typing entry after a sent message or a disconnect.
    pub async fn clear_actor_typing(self: &Arc<Self>, actor_key: &ChatActorKey) {
        if self.cache.clear_typing(actor_key).await {
            self.schedule_typing_broadcast();
        }
    }

    /// Schedule one broadcast after the coalescing window unless one is
    /// already pending. At most one task exists at a time.
    fn schedule_typing_broadcast(self: &Arc<Self>) {
        if !self.cache.claim_typing_broadcast() {
            return;
        }
        let service = Arc::clone(self);
        tokio::spawn(async move {
            tokio::time::sleep(TYPING_BROADCAST_COALESCE).await;
            service.cache.release_typing_broadcast();
            let now = Utc::now();
            let actors = service
                .cache
                .active_typing_actors(now, LIVE_CHAT_TYPING_BROADCAST_MAX_ACTORS)
                .await;
            service.cache.record_typing_broadcast(now);
            service.cache.broadcast(LiveChatServerEvent::TypingSet {
                actors,
                expires_at: now + LIVE_CHAT_TYPING_TTL,
            });
        });
    }
}
