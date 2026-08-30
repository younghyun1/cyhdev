//! Fixed-capacity one-time state for single-process OIDC authorization flows.

use std::{
    fmt,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use zeroize::Zeroizing;

use crate::features::accounts::{
    domain::oidc::{OidcFlowMode, OidcIdentityClaims},
    error::AccountError,
};

pub const MAX_PENDING_OIDC_FLOWS: usize = 512;
const OIDC_FLOW_TTL: Duration = Duration::from_secs(10 * 60);
const OIDC_LINK_COMPLETION_TTL: Duration = Duration::from_secs(5 * 60);
const TOKEN_SECRET_BYTES: usize = 32;
const TOKEN_LENGTH: usize = 43;
const TOKEN_DECODE_BUFFER_BYTES: usize = 33;

pub(super) struct PendingAuthorization {
    pub(super) mode: OidcFlowMode,
    pub(super) pkce_verifier: Zeroizing<String>,
    pub(super) nonce: Zeroizing<String>,
}

pub(super) struct CompletedLink {
    pub(super) expected_user_id: Uuid,
    pub(super) identity: OidcIdentityClaims,
}

pub(crate) struct OneTimeToken(Zeroizing<String>);

impl OneTimeToken {
    pub(crate) fn expose(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Debug for OneTimeToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("OneTimeToken([REDACTED])")
    }
}

pub(super) struct OidcFlowStores {
    pending: OneTimeStore<PendingAuthorization>,
    completed_links: OneTimeStore<CompletedLink>,
}

impl Default for OidcFlowStores {
    fn default() -> Self {
        Self {
            pending: OneTimeStore::new(MAX_PENDING_OIDC_FLOWS, OIDC_FLOW_TTL),
            completed_links: OneTimeStore::new(MAX_PENDING_OIDC_FLOWS, OIDC_LINK_COMPLETION_TTL),
        }
    }
}

impl OidcFlowStores {
    pub(super) async fn insert_pending(
        &self,
        flow: PendingAuthorization,
    ) -> Result<OneTimeToken, AccountError> {
        self.pending.insert(flow).await
    }

    pub(super) async fn take_pending(&self, token: &str) -> Option<PendingAuthorization> {
        self.pending.take(token).await
    }

    pub(super) async fn insert_completed_link(
        &self,
        link: CompletedLink,
    ) -> Result<OneTimeToken, AccountError> {
        self.completed_links.insert(link).await
    }

    pub(super) async fn take_completed_link(&self, token: &str) -> Option<CompletedLink> {
        self.completed_links.take(token).await
    }
}

struct OneTimeStore<T> {
    entries: scc::HashMap<FlowKey, Expiring<T>>,
    active_slots: AtomicUsize,
    max_entries: usize,
    ttl: Duration,
}

impl<T> OneTimeStore<T> {
    fn new(max_entries: usize, ttl: Duration) -> Self {
        Self {
            entries: scc::HashMap::with_capacity(max_entries),
            active_slots: AtomicUsize::new(0),
            max_entries,
            ttl,
        }
    }

    async fn insert(&self, value: T) -> Result<OneTimeToken, AccountError> {
        if !self.try_reserve_slot() {
            self.purge_expired().await;
            if !self.try_reserve_slot() {
                return Err(AccountError::OidcFlowStoreSaturated {
                    max_flows: self.max_entries,
                });
            }
        }

        let mut secret = Zeroizing::new([0_u8; TOKEN_SECRET_BYTES]);
        if let Err(error) = getrandom::fill(secret.as_mut()) {
            self.release_slot();
            return Err(AccountError::OidcFlowEntropy(error));
        }
        let key = FlowKey::from_secret(secret.as_ref());
        let token = OneTimeToken(Zeroizing::new(URL_SAFE_NO_PAD.encode(secret.as_ref())));
        let entry = Expiring {
            value,
            expires_at: Instant::now() + self.ttl,
        };
        match self.entries.insert_async(key, entry).await {
            Ok(()) => Ok(token),
            Err(_) => {
                self.release_slot();
                Err(AccountError::OidcFlowRejected)
            }
        }
    }

    async fn take(&self, token: &str) -> Option<T> {
        let key = FlowKey::from_token(token)?;
        let (_, entry) = self.entries.remove_async(&key).await?;
        self.release_slot();
        (entry.expires_at > Instant::now()).then_some(entry.value)
    }

    async fn purge_expired(&self) {
        let now = Instant::now();
        self.entries
            .iter_mut_async(|entry| {
                if entry.expires_at <= now {
                    let _ = entry.consume();
                    self.release_slot();
                }
                true
            })
            .await;
    }

    fn try_reserve_slot(&self) -> bool {
        self.active_slots
            .try_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                (current < self.max_entries).then_some(current + 1)
            })
            .is_ok()
    }

    fn release_slot(&self) {
        self.active_slots.fetch_sub(1, Ordering::AcqRel);
    }
}

struct Expiring<T> {
    value: T,
    expires_at: Instant,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
struct FlowKey([u8; TOKEN_SECRET_BYTES]);

impl FlowKey {
    fn from_secret(secret: &[u8]) -> Self {
        Self(Sha256::digest(secret).into())
    }

    fn from_token(token: &str) -> Option<Self> {
        if token.len() != TOKEN_LENGTH {
            return None;
        }
        let mut secret = Zeroizing::new([0_u8; TOKEN_DECODE_BUFFER_BYTES]);
        match URL_SAFE_NO_PAD.decode_slice(token.as_bytes(), secret.as_mut()) {
            Ok(TOKEN_SECRET_BYTES) => Some(Self::from_secret(&secret[..TOKEN_SECRET_BYTES])),
            Ok(_) | Err(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn one_time_tokens_are_consumed_exactly_once() -> Result<(), AccountError> {
        let store = OneTimeStore::new(2, Duration::from_secs(60));
        let token = store.insert(7_u8).await?;
        assert_eq!(store.take(token.expose()).await, Some(7));
        assert_eq!(store.take(token.expose()).await, None);
        assert_eq!(format!("{token:?}"), "OneTimeToken([REDACTED])");
        Ok(())
    }

    #[tokio::test]
    async fn active_capacity_fails_closed_without_eviction() -> Result<(), AccountError> {
        let store = OneTimeStore::new(1, Duration::from_secs(60));
        let first = store.insert(1_u8).await?;
        assert!(matches!(
            store.insert(2_u8).await,
            Err(AccountError::OidcFlowStoreSaturated { max_flows: 1 })
        ));
        assert_eq!(store.take(first.expose()).await, Some(1));
        Ok(())
    }

    #[tokio::test]
    async fn expired_tokens_are_rejected() -> Result<(), AccountError> {
        let store = OneTimeStore::new(1, Duration::ZERO);
        let token = store.insert(1_u8).await?;
        assert_eq!(store.take(token.expose()).await, None);
        Ok(())
    }
}
