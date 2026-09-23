//! Bounded typing state and the bookkeeping behind coalesced typing broadcasts.

use std::sync::atomic::Ordering;

use chrono::{DateTime, Utc};

use super::{ChatActor, ChatActorKey, LIVE_CHAT_TYPING_MAX_ENTRIES, LiveChatCache, TypingState};

impl LiveChatCache {
    /// Insert or refresh an actor's typing entry. Returns true only when the
    /// actor was not already typing, i.e. when the visible set changed.
    pub async fn set_typing(&self, mut state: TypingState) -> bool {
        let actor_key = state.actor.actor_key.clone();
        if let ChatActorKey::User(user_id) = &actor_key
            && self.is_connected_user_disabled(*user_id)
        {
            let _ = state.actor.anonymize_deleted_user(*user_id);
        }
        if self
            .typing_by_actor
            .update_async(&actor_key, |_, current| *current = state.clone())
            .await
            .is_some()
        {
            return false;
        }
        if self
            .typing_count
            .try_update(Ordering::SeqCst, Ordering::SeqCst, |current| {
                (current < LIVE_CHAT_TYPING_MAX_ENTRIES).then_some(current + 1)
            })
            .is_err()
        {
            return false;
        }
        match self.typing_by_actor.insert_async(actor_key, state).await {
            Ok(_) => true,
            Err(_) => {
                self.typing_count.fetch_sub(1, Ordering::SeqCst);
                false
            }
        }
    }

    pub async fn clear_typing(&self, actor_key: &ChatActorKey) -> bool {
        if self.typing_by_actor.remove_async(actor_key).await.is_some() {
            self.typing_count.fetch_sub(1, Ordering::SeqCst);
            true
        } else {
            false
        }
    }

    pub async fn clear_expired_typing(&self, now: DateTime<Utc>) {
        let mut removed = 0usize;
        self.typing_by_actor
            .retain_async(|_, typing_state| {
                let retain = typing_state.expires_at > now;
                if !retain {
                    removed = removed.saturating_add(1);
                }
                retain
            })
            .await;
        self.typing_count.fetch_sub(removed, Ordering::SeqCst);
    }

    /// Unexpired typing actors, at most `limit` of them. Browsers show two
    /// names and a count, so broadcasting the full 4,096-entry set would only
    /// multiply payload size by the number of recipients.
    pub async fn active_typing_actors(&self, now: DateTime<Utc>, limit: usize) -> Vec<ChatActor> {
        self.clear_expired_typing(now).await;
        let mut actors = Vec::with_capacity(limit.min(self.typing_by_actor.len()));
        self.typing_by_actor
            .iter_async(|_, typing_state| {
                if actors.len() >= limit {
                    return false;
                }
                actors.push(typing_state.actor.clone());
                true
            })
            .await;
        actors
    }

    /// Claim the single pending typing broadcast. False means one is already
    /// scheduled and will observe the caller's change when it runs.
    pub fn claim_typing_broadcast(&self) -> bool {
        self.typing_broadcast_pending
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    /// Release the claim before snapshotting, so a change racing the snapshot
    /// schedules another broadcast instead of being lost.
    pub fn release_typing_broadcast(&self) {
        self.typing_broadcast_pending
            .store(false, Ordering::Release);
    }

    pub fn record_typing_broadcast(&self, now: DateTime<Utc>) {
        self.typing_broadcast_at_millis
            .store(now.timestamp_millis(), Ordering::Release);
    }

    /// Whether the last typing broadcast is at least `interval_millis` old.
    pub fn typing_broadcast_older_than(&self, now: DateTime<Utc>, interval_millis: i64) -> bool {
        let last = self.typing_broadcast_at_millis.load(Ordering::Acquire);
        now.timestamp_millis().saturating_sub(last) >= interval_millis
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use chrono::Duration;

    use super::*;
    use crate::features::live_chat::domain::guest_identity::GuestIdentityKey;

    fn typing(index: u8, expires_at: DateTime<Utc>) -> TypingState {
        let key = GuestIdentityKey::from_secret(&[0x11; 32]);
        TypingState {
            actor: ChatActor::guest(IpAddr::V4(Ipv4Addr::new(192, 0, 2, index)), &key, None),
            room_key: "main".to_owned(),
            expires_at,
        }
    }

    #[tokio::test]
    async fn only_new_typers_change_the_set_and_payloads_are_capped() {
        let cache = LiveChatCache::new(1024);
        let expires_at = Utc::now() + Duration::seconds(4);
        for index in 1..=20 {
            assert!(cache.set_typing(typing(index, expires_at)).await);
        }
        // A refresh updates the expiry without changing the visible set.
        assert!(!cache.set_typing(typing(1, expires_at)).await);
        assert_eq!(cache.active_typing_actors(Utc::now(), 8).await.len(), 8);
        assert_eq!(cache.active_typing_actors(Utc::now(), 64).await.len(), 20);
    }

    #[test]
    fn one_broadcast_claim_is_outstanding_at_a_time() {
        let cache = LiveChatCache::new(1024);
        assert!(cache.claim_typing_broadcast());
        assert!(!cache.claim_typing_broadcast());
        cache.release_typing_broadcast();
        assert!(cache.claim_typing_broadcast());
        let now = Utc::now();
        cache.record_typing_broadcast(now);
        assert!(!cache.typing_broadcast_older_than(now, 2_000));
        assert!(cache.typing_broadcast_older_than(now + Duration::seconds(2), 2_000));
    }
}
