//! Bounded per-second message-rate windows used for abuse detection.

use std::sync::atomic::Ordering;

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use uuid::Uuid;

use super::{
    LIVE_CHAT_ABNORMAL_MESSAGE_LIMIT_PER_SECOND, LIVE_CHAT_RATE_MAX_ENTRIES, LiveChatCache,
};
use crate::features::live_chat::domain::ip_prefix::LiveChatIpPrefix;

/// Windows older than this carry no signal for a one-second limit; the short
/// sweep removes them so a burst of distinct senders frees capacity quickly.
const STALE_RATE_WINDOW: ChronoDuration = ChronoDuration::seconds(2);
/// Retention for the minute-level sweep, which only bounds idle growth.
const EXPIRED_RATE_WINDOW: ChronoDuration = ChronoDuration::seconds(60);

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub(super) enum LiveChatRateKey {
    User(Uuid),
    /// IPv4 exact, IPv6 grouped by /64; see [`LiveChatIpPrefix`].
    Ip(LiveChatIpPrefix),
}

#[derive(Debug, Clone)]
pub(super) struct LiveChatRateState {
    pub(super) window_started_at: DateTime<Utc>,
    pub(super) count: u32,
}

/// Outcome of recording one chat message attempt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MessageRateDecision {
    Allowed,
    /// More than the abnormal limit within one second; the caller bans.
    Abnormal,
    /// The bounded table had no room for a new sender. The caller rejects the
    /// message transiently: capacity pressure is not evidence of abuse.
    Saturated,
}

impl LiveChatCache {
    /// Count one message for the sender's address group and, when signed in,
    /// their account. Abnormal wins over saturation so a known abuser is still
    /// banned while the table is full.
    pub async fn record_message_attempt(
        &self,
        user_id: Option<Uuid>,
        ip: std::net::IpAddr,
        now: DateTime<Utc>,
    ) -> MessageRateDecision {
        let ip_decision = self
            .record_message_attempt_for_key(LiveChatRateKey::Ip(LiveChatIpPrefix::of(ip)), now)
            .await;
        let user_decision = match user_id {
            Some(user_id) => {
                self.record_message_attempt_for_key(LiveChatRateKey::User(user_id), now)
                    .await
            }
            None => MessageRateDecision::Allowed,
        };
        match (ip_decision, user_decision) {
            (MessageRateDecision::Abnormal, _) | (_, MessageRateDecision::Abnormal) => {
                MessageRateDecision::Abnormal
            }
            (MessageRateDecision::Saturated, _) | (_, MessageRateDecision::Saturated) => {
                MessageRateDecision::Saturated
            }
            (MessageRateDecision::Allowed, MessageRateDecision::Allowed) => {
                MessageRateDecision::Allowed
            }
        }
    }

    async fn record_message_attempt_for_key(
        &self,
        key: LiveChatRateKey,
        now: DateTime<Utc>,
    ) -> MessageRateDecision {
        if let Some(decision) = self.count_existing_window(&key, now).await {
            return decision;
        }
        if self
            .message_rate_count
            .try_update(Ordering::SeqCst, Ordering::SeqCst, |current| {
                (current < LIVE_CHAT_RATE_MAX_ENTRIES).then_some(current + 1)
            })
            .is_err()
        {
            return MessageRateDecision::Saturated;
        }
        let state = LiveChatRateState {
            window_started_at: now,
            count: 1,
        };
        match self.message_rate_by_key.insert_async(key, state).await {
            Ok(()) => MessageRateDecision::Allowed,
            Err((key, _state)) => {
                // A concurrent attempt inserted the key first; count against it.
                self.message_rate_count.fetch_sub(1, Ordering::SeqCst);
                match self.count_existing_window(&key, now).await {
                    Some(decision) => decision,
                    None => MessageRateDecision::Allowed,
                }
            }
        }
    }

    async fn count_existing_window(
        &self,
        key: &LiveChatRateKey,
        now: DateTime<Utc>,
    ) -> Option<MessageRateDecision> {
        self.message_rate_by_key
            .update_async(key, |_, state| {
                if now.signed_duration_since(state.window_started_at) < ChronoDuration::seconds(1) {
                    state.count = state.count.saturating_add(1);
                } else {
                    state.window_started_at = now;
                    state.count = 1;
                }
                if state.count > LIVE_CHAT_ABNORMAL_MESSAGE_LIMIT_PER_SECOND {
                    MessageRateDecision::Abnormal
                } else {
                    MessageRateDecision::Allowed
                }
            })
            .await
    }

    /// Short-interval sweep, run every second by the maintenance scheduler.
    pub async fn clear_stale_rate_windows(&self, now: DateTime<Utc>) {
        self.clear_rate_windows_before(now, STALE_RATE_WINDOW).await;
    }

    pub async fn clear_expired_rate_windows(&self, now: DateTime<Utc>) {
        self.clear_rate_windows_before(now, EXPIRED_RATE_WINDOW)
            .await;
    }

    async fn clear_rate_windows_before(&self, now: DateTime<Utc>, age: ChronoDuration) {
        let mut removed = 0usize;
        self.message_rate_by_key
            .retain_async(|_, state| {
                let retain = now.signed_duration_since(state.window_started_at) < age;
                if !retain {
                    removed = removed.saturating_add(1);
                }
                retain
            })
            .await;
        self.message_rate_count.fetch_sub(removed, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    use super::*;

    fn cache_with_rate_capacity(capacity: usize) -> LiveChatCache {
        let cache = LiveChatCache::new(1024);
        // Consume all but `capacity` slots so saturation is reachable quickly.
        cache
            .message_rate_count
            .store(LIVE_CHAT_RATE_MAX_ENTRIES - capacity, Ordering::SeqCst);
        cache
    }

    fn v6(last: u16, prefix: u16) -> IpAddr {
        IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, prefix, 0, 0, 0, last))
    }

    #[tokio::test]
    async fn full_table_rejects_new_senders_without_marking_them_abnormal() {
        let cache = cache_with_rate_capacity(1);
        let now = Utc::now();
        let first = IpAddr::V4(Ipv4Addr::new(198, 51, 100, 1));
        let second = IpAddr::V4(Ipv4Addr::new(198, 51, 100, 2));
        assert_eq!(
            cache.record_message_attempt(None, first, now).await,
            MessageRateDecision::Allowed
        );
        assert_eq!(
            cache.record_message_attempt(None, second, now).await,
            MessageRateDecision::Saturated
        );
        // Existing senders keep being counted while the table is full.
        assert_eq!(
            cache.record_message_attempt(None, first, now).await,
            MessageRateDecision::Allowed
        );
    }

    #[tokio::test]
    async fn stale_sweep_frees_capacity_for_new_senders() {
        let cache = cache_with_rate_capacity(1);
        let start = Utc::now();
        let first = IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1));
        let second = IpAddr::V4(Ipv4Addr::new(203, 0, 113, 2));
        let _ = cache.record_message_attempt(None, first, start).await;
        let later = start + ChronoDuration::seconds(3);
        cache.clear_stale_rate_windows(later).await;
        assert_eq!(
            cache.record_message_attempt(None, second, later).await,
            MessageRateDecision::Allowed
        );
    }

    #[tokio::test]
    async fn ipv6_senders_share_one_window_per_subscriber_prefix() {
        let cache = LiveChatCache::new(1024);
        let now = Utc::now();
        for host in 0..LIVE_CHAT_ABNORMAL_MESSAGE_LIMIT_PER_SECOND {
            assert_eq!(
                cache
                    .record_message_attempt(None, v6(host as u16 + 1, 7), now)
                    .await,
                MessageRateDecision::Allowed
            );
        }
        assert_eq!(
            cache.record_message_attempt(None, v6(999, 7), now).await,
            MessageRateDecision::Abnormal
        );
        assert_eq!(
            cache.record_message_attempt(None, v6(1, 8), now).await,
            MessageRateDecision::Allowed
        );
        assert_eq!(cache.message_rate_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn abnormal_wins_over_saturation() {
        let cache = cache_with_rate_capacity(1);
        let now = Utc::now();
        let user = Uuid::now_v7();
        let ip = IpAddr::V4(Ipv4Addr::new(192, 0, 2, 9));
        // The address key takes the last slot, so the account key saturates
        // on every attempt while the address window still detects the burst.
        let mut last = MessageRateDecision::Allowed;
        for _ in 0..=LIVE_CHAT_ABNORMAL_MESSAGE_LIMIT_PER_SECOND {
            last = cache.record_message_attempt(Some(user), ip, now).await;
        }
        assert_eq!(last, MessageRateDecision::Abnormal);
    }
}
