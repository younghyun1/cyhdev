//! Bounded post-commit identity invalidation for retained live-chat state.

use std::{sync::Arc, sync::atomic::Ordering};

use tracing::error;
use uuid::Uuid;

use crate::features::live_chat::domain::rtc::RtcServerSignal;

use super::{ChatActorKey, LIVE_CHAT_MAX_CONNECTIONS, LiveChatCache, LiveChatServerEvent};

impl LiveChatCache {
    /// Rewrite every cached presentation of one deleted account.
    ///
    /// The disabled-user set contains only users with live connections, so its
    /// maximum cardinality is the fixed connection limit. Call this only after
    /// the account deletion transaction commits.
    pub async fn anonymize_deleted_user(&self, user_id: Uuid) {
        let _identity_guard = self.identity_mutation.lock().await;

        let mut connection_ids = Vec::new();
        self.connected_clients
            .iter_async(|connection_id, connection| {
                if connection.authority_user_id == Some(user_id) {
                    connection_ids.push(*connection_id);
                }
                true
            })
            .await;

        if !connection_ids.is_empty() && !self.disabled_connected_users.contains_sync(&user_id) {
            // One distinct disabled user requires at least one connection, so
            // the connection bound proves this set cannot exceed its capacity.
            if self.disabled_connected_users.len() < LIVE_CHAT_MAX_CONNECTIONS as usize {
                let _ = self.disabled_connected_users.insert_async(user_id).await;
            } else {
                // This should be unreachable while connection accounting is
                // correct. Fail closed without allowing the set to grow.
                self.disabled_connected_users_saturated
                    .store(true, Ordering::SeqCst);
                error!(
                    user_id = %user_id,
                    max_entries = LIVE_CHAT_MAX_CONNECTIONS,
                    "Live-chat disabled-user index saturated; rejecting all registered users"
                );
            }
        }

        for connection_id in connection_ids {
            let _ = self
                .connected_clients
                .update_async(&connection_id, |_, connection| {
                    let _ = connection.actor.anonymize_deleted_user(user_id);
                    let _ = connection.disconnect_tx.send(true);
                })
                .await;
        }

        let _ = self
            .typing_by_actor
            .update_async(&ChatActorKey::User(user_id), |_, typing| {
                let _ = typing.actor.anonymize_deleted_user(user_id);
            })
            .await;

        self.anonymize_cached_messages(user_id).await;
    }

    async fn anonymize_cached_messages(&self, user_id: Uuid) {
        let mut eviction_queue = self.eviction_queue.lock().await;
        let mut message_ids = Vec::new();
        self.messages_by_id
            .iter_async(|message_id, message| {
                if message.user_id == Some(user_id) {
                    message_ids.push(*message_id);
                }
                true
            })
            .await;

        for message_id in message_ids {
            let mut byte_sizes = None;
            let _ = self
                .messages_by_id
                .update_async(&message_id, |_, current| {
                    let previous_bytes = current.estimated_bytes();
                    let mut anonymized = (**current).clone();
                    if anonymized.anonymize_deleted_user(user_id) {
                        let retained_bytes = anonymized.estimated_bytes();
                        *current = Arc::new(anonymized);
                        byte_sizes = Some((previous_bytes, retained_bytes));
                    }
                })
                .await;

            let Some((previous_bytes, retained_bytes)) = byte_sizes else {
                continue;
            };
            if retained_bytes >= previous_bytes {
                self.total_bytes
                    .fetch_add(retained_bytes - previous_bytes, Ordering::SeqCst);
            } else {
                self.total_bytes
                    .fetch_sub(previous_bytes - retained_bytes, Ordering::SeqCst);
            }
            for entry in eviction_queue.iter_mut() {
                if entry.live_chat_message_id == message_id {
                    entry.estimated_bytes = retained_bytes;
                    break;
                }
            }
        }
        self.evict_over_budget(&mut eviction_queue).await;
    }

    /// Expected O(1) check used by message and RTC paths after session revocation.
    pub fn is_connected_user_disabled(&self, user_id: Uuid) -> bool {
        self.disabled_connected_users_saturated
            .load(Ordering::SeqCst)
            || self.disabled_connected_users.contains_sync(&user_id)
    }

    /// Apply current deletion markers immediately before public serialization.
    pub fn anonymize_event_for_public(&self, event: &mut LiveChatServerEvent) {
        match event {
            LiveChatServerEvent::Hello {
                actor,
                recent_messages,
                ..
            } => {
                self.anonymize_actor_if_disabled(actor);
                for message in recent_messages {
                    self.anonymize_message_if_disabled(message);
                }
            }
            LiveChatServerEvent::Message { message }
            | LiveChatServerEvent::MessageAck { message, .. } => {
                self.anonymize_message_if_disabled(message);
            }
            LiveChatServerEvent::Typing { actor, .. } => {
                self.anonymize_actor_if_disabled(actor);
            }
            LiveChatServerEvent::TypingSet { actors, .. } => {
                for actor in actors {
                    self.anonymize_actor_if_disabled(actor);
                }
            }
            LiveChatServerEvent::Rtc(signal) => self.anonymize_rtc_signal(signal),
            LiveChatServerEvent::Presence { .. }
            | LiveChatServerEvent::HeartbeatAck { .. }
            | LiveChatServerEvent::Error { .. } => {}
        }
    }

    fn anonymize_actor_if_disabled(&self, actor: &mut super::ChatActor) {
        let Some(user_id) = actor.user_id else {
            return;
        };
        if self.is_connected_user_disabled(user_id) {
            let _ = actor.anonymize_deleted_user(user_id);
        }
    }

    fn anonymize_message_if_disabled(&self, message: &mut super::CachedChatMessage) {
        let Some(user_id) = message.user_id else {
            return;
        };
        if self.is_connected_user_disabled(user_id) {
            let _ = message.anonymize_deleted_user(user_id);
        }
    }

    fn anonymize_rtc_signal(&self, signal: &mut RtcServerSignal) {
        match signal {
            RtcServerSignal::PeerState { actor, .. } => {
                self.anonymize_actor_if_disabled(actor);
            }
            RtcServerSignal::Roster { participants } => {
                for participant in participants {
                    self.anonymize_actor_if_disabled(&mut participant.actor);
                }
            }
            RtcServerSignal::Answer { .. }
            | RtcServerSignal::Offer { .. }
            | RtcServerSignal::Ice(_)
            | RtcServerSignal::Error { .. } => {}
        }
    }
}
