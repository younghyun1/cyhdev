//! Bounded live-connection registry with per-address-group caps.

use std::sync::atomic::Ordering;

use uuid::Uuid;

use super::{ChatConnectionState, LIVE_CHAT_MAX_CONNECTIONS, LiveChatCache};
use crate::features::live_chat::domain::ip_prefix::LiveChatIpPrefix;

/// Concurrent connections allowed per IPv4 address or IPv6 /64. Enough for a
/// household sharing one address with several tabs each, while one host can
/// no longer hold the whole 4,096-connection budget with idle sockets.
pub const LIVE_CHAT_MAX_CONNECTIONS_PER_ADDRESS: usize = 8;

/// Outcome of registering an upgraded connection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectionAdmission {
    Admitted,
    /// The authority account was deleted while connected.
    Disabled,
    /// The process-wide connection budget is exhausted.
    Full,
    /// The sender's address group already holds its share of connections.
    AddressLimit,
}

impl LiveChatCache {
    pub async fn register_connection(
        &self,
        connection_id: Uuid,
        connection_state: ChatConnectionState,
    ) -> ConnectionAdmission {
        let _identity_guard = self.identity_mutation.lock().await;
        if let Some(user_id) = connection_state.authority_user_id
            && self.is_connected_user_disabled(user_id)
        {
            return ConnectionAdmission::Disabled;
        }
        if self
            .connected_count
            .try_update(Ordering::SeqCst, Ordering::SeqCst, |current| {
                (current < LIVE_CHAT_MAX_CONNECTIONS).then_some(current + 1)
            })
            .is_err()
        {
            return ConnectionAdmission::Full;
        }
        let prefix = connection_state.client_prefix;
        if !self.reserve_prefix_slot(prefix).await {
            self.connected_count.fetch_sub(1, Ordering::SeqCst);
            return ConnectionAdmission::AddressLimit;
        }
        match self
            .connected_clients
            .insert_async(connection_id, connection_state)
            .await
        {
            Ok(_) => ConnectionAdmission::Admitted,
            Err(_) => {
                self.release_prefix_slot(prefix).await;
                self.connected_count.fetch_sub(1, Ordering::SeqCst);
                ConnectionAdmission::Full
            }
        }
    }

    pub async fn unregister_connection(&self, connection_id: Uuid) {
        let _identity_guard = self.identity_mutation.lock().await;
        let Some((_, removed_connection)) =
            self.connected_clients.remove_async(&connection_id).await
        else {
            return;
        };
        self.connected_count.fetch_sub(1, Ordering::SeqCst);
        self.release_prefix_slot(removed_connection.client_prefix)
            .await;

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

    /// Caller holds `identity_mutation`, which serializes every prefix update.
    async fn reserve_prefix_slot(&self, prefix: LiveChatIpPrefix) -> bool {
        let existing = self
            .connections_by_prefix
            .update_async(&prefix, |_, count| {
                if *count >= LIVE_CHAT_MAX_CONNECTIONS_PER_ADDRESS {
                    false
                } else {
                    *count += 1;
                    true
                }
            })
            .await;
        match existing {
            Some(reserved) => reserved,
            None => self
                .connections_by_prefix
                .insert_async(prefix, 1)
                .await
                .is_ok(),
        }
    }

    /// Caller holds `identity_mutation`.
    async fn release_prefix_slot(&self, prefix: LiveChatIpPrefix) {
        let remaining = self
            .connections_by_prefix
            .update_async(&prefix, |_, count| {
                *count = count.saturating_sub(1);
                *count
            })
            .await;
        if remaining == Some(0) {
            let _ = self
                .connections_by_prefix
                .remove_if_async(&prefix, |count| *count == 0)
                .await;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv6Addr};

    use chrono::Utc;

    use super::*;
    use crate::features::live_chat::domain::{actor::ChatActor, guest_identity::GuestIdentityKey};

    fn state(ip: IpAddr) -> ChatConnectionState {
        let key = GuestIdentityKey::from_secret(&[0x22; 32]);
        ChatConnectionState {
            actor: ChatActor::guest(ip, &key, None),
            authority_user_id: None,
            client_prefix: LiveChatIpPrefix::of(ip),
            disconnect_tx: tokio::sync::watch::channel(false).0,
            room_key: "main".to_owned(),
            connected_at: Utc::now(),
        }
    }

    fn v6(prefix: u16, host: u16) -> IpAddr {
        IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, prefix, 0, 0, 0, host))
    }

    #[tokio::test]
    async fn address_groups_are_capped_and_released() {
        let cache = LiveChatCache::new(1024);
        let mut admitted = Vec::new();
        for host in 0..LIVE_CHAT_MAX_CONNECTIONS_PER_ADDRESS {
            let id = Uuid::now_v7();
            let ip = v6(1, host as u16 + 1);
            assert_eq!(
                cache.register_connection(id, state(ip)).await,
                ConnectionAdmission::Admitted
            );
            admitted.push(id);
        }
        assert_eq!(
            cache
                .register_connection(Uuid::now_v7(), state(v6(1, 999)))
                .await,
            ConnectionAdmission::AddressLimit
        );
        assert_eq!(
            cache
                .register_connection(Uuid::now_v7(), state(v6(2, 1)))
                .await,
            ConnectionAdmission::Admitted
        );
        assert_eq!(
            cache.connected_count(),
            LIVE_CHAT_MAX_CONNECTIONS_PER_ADDRESS as u64 + 1
        );

        for id in &admitted {
            cache.unregister_connection(*id).await;
        }
        assert!(
            cache
                .connections_by_prefix
                .read_async(&LiveChatIpPrefix::of(v6(1, 1)), |_, count| *count)
                .await
                .is_none()
        );
        assert_eq!(
            cache
                .register_connection(Uuid::now_v7(), state(v6(1, 999)))
                .await,
            ConnectionAdmission::Admitted
        );
    }
}
