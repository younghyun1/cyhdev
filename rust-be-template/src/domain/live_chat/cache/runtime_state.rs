use std::{net::IpAddr, sync::atomic::Ordering};

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use uuid::Uuid;

use super::{
    ChatActor, ChatActorKey, ChatConnectionState, LIVE_CHAT_ABNORMAL_MESSAGE_LIMIT_PER_SECOND,
    LIVE_CHAT_MAX_CONNECTIONS, LIVE_CHAT_RATE_MAX_ENTRIES, LIVE_CHAT_TYPING_MAX_ENTRIES,
    LiveChatCache, TypingState,
    rate::{LiveChatRateKey, LiveChatRateState},
};

impl LiveChatCache {
    pub async fn record_message_attempt(
        &self,
        user_id: Option<Uuid>,
        ip: IpAddr,
        now: DateTime<Utc>,
    ) -> bool {
        let ip_abnormal = self
            .record_message_attempt_for_key(LiveChatRateKey::Ip(ip), now)
            .await;
        let user_abnormal = match user_id {
            Some(user_id) => {
                self.record_message_attempt_for_key(LiveChatRateKey::User(user_id), now)
                    .await
            }
            None => false,
        };
        ip_abnormal || user_abnormal
    }

    async fn record_message_attempt_for_key(
        &self,
        key: LiveChatRateKey,
        now: DateTime<Utc>,
    ) -> bool {
        let mut is_abnormal = false;
        if self
            .message_rate_by_key
            .update_async(&key, |_, state| {
                if now.signed_duration_since(state.window_started_at) < ChronoDuration::seconds(1) {
                    state.count = state.count.saturating_add(1);
                } else {
                    state.window_started_at = now;
                    state.count = 1;
                }
                is_abnormal = state.count > LIVE_CHAT_ABNORMAL_MESSAGE_LIMIT_PER_SECOND;
            })
            .await
            .is_none()
        {
            if self
                .message_rate_count
                .try_update(Ordering::SeqCst, Ordering::SeqCst, |current| {
                    (current < LIVE_CHAT_RATE_MAX_ENTRIES).then_some(current + 1)
                })
                .is_err()
            {
                return true;
            }
            match self
                .message_rate_by_key
                .insert_async(
                    key,
                    LiveChatRateState {
                        window_started_at: now,
                        count: 1,
                    },
                )
                .await
            {
                Ok(_) => {}
                Err((key, _state)) => {
                    self.message_rate_count.fetch_sub(1, Ordering::SeqCst);
                    let mut raced_is_abnormal = false;
                    let _ = self
                        .message_rate_by_key
                        .update_async(&key, |_, state| {
                            if now.signed_duration_since(state.window_started_at)
                                < ChronoDuration::seconds(1)
                            {
                                state.count = state.count.saturating_add(1);
                            } else {
                                state.window_started_at = now;
                                state.count = 1;
                            }
                            raced_is_abnormal =
                                state.count > LIVE_CHAT_ABNORMAL_MESSAGE_LIMIT_PER_SECOND;
                        })
                        .await;
                    return raced_is_abnormal;
                }
            }
        }
        is_abnormal
    }

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

    pub async fn clear_stale_rate_windows(&self, now: DateTime<Utc>) {
        self.clear_rate_windows_before(now, ChronoDuration::seconds(2))
            .await;
    }

    pub async fn clear_expired_rate_windows(&self, now: DateTime<Utc>) {
        self.clear_rate_windows_before(now, ChronoDuration::seconds(60))
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
        self.message_rate_count
            .fetch_sub(removed, Ordering::SeqCst);
    }

    pub async fn active_typing_actors(&self, now: DateTime<Utc>) -> Vec<ChatActor> {
        self.clear_expired_typing(now).await;
        let mut actors = Vec::with_capacity(self.typing_by_actor.len());
        self.typing_by_actor
            .iter_async(|_, typing_state| {
                actors.push(typing_state.actor.clone());
                true
            })
            .await;
        actors
    }

    pub async fn register_connection(
        &self,
        connection_id: Uuid,
        connection_state: ChatConnectionState,
    ) -> bool {
        let _identity_guard = self.identity_mutation.lock().await;
        if let Some(user_id) = connection_state.authority_user_id
            && self.is_connected_user_disabled(user_id)
        {
            return false;
        }
        if self
            .connected_count
            .try_update(Ordering::SeqCst, Ordering::SeqCst, |current| {
                (current < LIVE_CHAT_MAX_CONNECTIONS).then_some(current + 1)
            })
            .is_err()
        {
            return false;
        }
        match self
            .connected_clients
            .insert_async(connection_id, connection_state)
            .await
        {
            Ok(_) => true,
            Err(_) => {
                self.connected_count.fetch_sub(1, Ordering::SeqCst);
                false
            }
        }
    }

    pub async fn unregister_connection(&self, connection_id: Uuid) {
        let _identity_guard = self.identity_mutation.lock().await;
        let Some((_, removed_connection)) = self.connected_clients.remove_async(&connection_id).await
        else {
            return;
        };
        self.connected_count.fetch_sub(1, Ordering::SeqCst);

        let Some(user_id) = removed_connection.authority_user_id else {
            return;
        };
        if !self.is_connected_user_disabled(user_id) {
            return;
        }
        let mut another_connection_exists = false;
        self.connected_clients
            .iter_async(|_, connection| {
                if connection.authority_user_id == Some(user_id) {
                    another_connection_exists = true;
                    return false;
                }
                true
            })
            .await;
        if !another_connection_exists {
            let _ = self.disabled_connected_users.remove_async(&user_id).await;
        }
    }

    pub fn connected_count(&self) -> u64 {
        self.connected_count.load(Ordering::SeqCst)
    }
}
