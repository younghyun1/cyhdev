//! Fixed-capacity, uniform-TTL one-time capability store.

use std::{
    collections::{BTreeMap, HashMap},
    fmt,
    time::{Duration, Instant},
};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::features::accounts::error::AccountError;

const TOKEN_SECRET_BYTES: usize = 32;
const TOKEN_LENGTH: usize = 43;
const TOKEN_DECODE_BUFFER_BYTES: usize = 33;

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

/// Draws a 256-bit secret as 43 unpadded base64url characters.
pub(super) fn random_token() -> Result<OneTimeToken, getrandom::Error> {
    let mut secret = Zeroizing::new([0_u8; TOKEN_SECRET_BYTES]);
    getrandom::fill(secret.as_mut())?;
    Ok(OneTimeToken(Zeroizing::new(
        URL_SAFE_NO_PAD.encode(secret.as_ref()),
    )))
}

/// One-time entries with a uniform TTL, so insertion order is also expiry order.
///
/// When full, the oldest entry is evicted rather than refusing new flows: a flood of starts
/// can then only cut short flows older than itself, and users simply start again, whereas
/// refusing would block every new login until the flood's entries expired.
pub(super) struct OneTimeStore<T> {
    inner: tokio::sync::Mutex<StoreInner<T>>,
    max_entries: usize,
    ttl: Duration,
}

struct StoreInner<T> {
    entries: HashMap<FlowKey, Expiring<T>>,
    order: BTreeMap<u64, FlowKey>,
    next_sequence: u64,
}

struct Expiring<T> {
    value: T,
    expires_at: Instant,
    sequence: u64,
}

impl<T> OneTimeStore<T> {
    pub(super) fn new(max_entries: usize, ttl: Duration) -> Self {
        Self {
            inner: tokio::sync::Mutex::new(StoreInner {
                entries: HashMap::with_capacity(max_entries),
                order: BTreeMap::new(),
                next_sequence: 0,
            }),
            max_entries,
            ttl,
        }
    }

    pub(super) async fn insert(&self, value: T) -> Result<OneTimeToken, AccountError> {
        let token = random_token().map_err(AccountError::OidcFlowEntropy)?;
        let key = match FlowKey::from_token(token.expose()) {
            Some(key) => key,
            None => return Err(AccountError::OidcFlowRejected),
        };
        let now = Instant::now();
        let mut inner = self.inner.lock().await;
        inner.purge_expired(now);
        if inner.entries.contains_key(&key) {
            return Err(AccountError::OidcFlowRejected);
        }
        if inner.entries.len() >= self.max_entries && inner.evict_oldest() {
            tracing::warn!(
                event = "oidc_flow_evicted",
                max_flows = self.max_entries,
                "Evicted the oldest pending OpenID Connect flow at capacity"
            );
        }
        let sequence = inner.next_sequence;
        inner.next_sequence = inner.next_sequence.wrapping_add(1);
        inner.order.insert(sequence, key);
        inner.entries.insert(
            key,
            Expiring {
                value,
                expires_at: now + self.ttl,
                sequence,
            },
        );
        Ok(token)
    }

    pub(super) async fn take(&self, token: &str) -> Option<T> {
        let key = FlowKey::from_token(token)?;
        let mut inner = self.inner.lock().await;
        let entry = inner.entries.remove(&key)?;
        inner.order.remove(&entry.sequence);
        (entry.expires_at > Instant::now()).then_some(entry.value)
    }
}

impl<T> StoreInner<T> {
    fn purge_expired(&mut self, now: Instant) {
        while let Some((&sequence, &key)) = self.order.first_key_value() {
            match self.entries.get(&key) {
                Some(entry) if entry.expires_at > now => break,
                _ => {
                    self.order.remove(&sequence);
                    self.entries.remove(&key);
                }
            }
        }
    }

    fn evict_oldest(&mut self) -> bool {
        match self.order.pop_first() {
            Some((_, key)) => self.entries.remove(&key).is_some(),
            None => false,
        }
    }
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
    async fn full_store_evicts_the_oldest_flow() -> Result<(), AccountError> {
        let store = OneTimeStore::new(2, Duration::from_secs(60));
        let first = store.insert(1_u8).await?;
        let second = store.insert(2_u8).await?;
        let third = store.insert(3_u8).await?;
        assert_eq!(store.take(first.expose()).await, None);
        assert_eq!(store.take(second.expose()).await, Some(2));
        assert_eq!(store.take(third.expose()).await, Some(3));
        let inner = store.inner.lock().await;
        assert!(inner.entries.is_empty() && inner.order.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn expired_tokens_are_rejected_and_purged() -> Result<(), AccountError> {
        let store = OneTimeStore::new(1, Duration::ZERO);
        let token = store.insert(1_u8).await?;
        assert_eq!(store.take(token.expose()).await, None);
        let _ = store.insert(2_u8).await?;
        let _ = store.insert(3_u8).await?;
        assert_eq!(store.inner.lock().await.entries.len(), 1);
        Ok(())
    }
}
