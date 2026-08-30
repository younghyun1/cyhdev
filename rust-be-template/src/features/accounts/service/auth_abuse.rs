//! Fixed-capacity process-local authentication throttles.

use std::{
    collections::HashMap,
    net::{IpAddr, Ipv6Addr},
    time::{Duration, Instant},
};

use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::features::accounts::domain::auth_abuse::{
    AuthAbusePruneReport, AuthEndpoint, AuthIdentity, AuthThrottleDimension,
    AuthThrottleRejection,
};
use crate::features::accounts::service::auth_abuse_policy::{
    FixedWindowLimit, identity_limits, ip_limits,
};

pub const MAX_AUTH_IP_RECORDS: usize = 16_384;
pub const MAX_AUTH_IDENTITY_RECORDS: usize = 32_768;
const AUTH_KEY_BYTES: usize = 32;
const CAPACITY_RETRY_AFTER: Duration = Duration::from_secs(60);

/// Owns the bounded authentication-abuse authority for one backend process.
pub struct AuthAbuseService {
    digest_key: Zeroizing<[u8; AUTH_KEY_BYTES]>,
    state: tokio::sync::Mutex<AuthAbuseState>,
    max_ip_records: usize,
    max_identity_records: usize,
}

impl AuthAbuseService {
    pub fn new() -> Result<Self, getrandom::Error> {
        let mut digest_key = Zeroizing::new([0_u8; AUTH_KEY_BYTES]);
        getrandom::fill(digest_key.as_mut())?;
        Ok(Self::with_limits(
            digest_key,
            MAX_AUTH_IP_RECORDS,
            MAX_AUTH_IDENTITY_RECORDS,
        ))
    }

    pub(super) fn with_limits(
        digest_key: Zeroizing<[u8; AUTH_KEY_BYTES]>,
        max_ip_records: usize,
        max_identity_records: usize,
    ) -> Self {
        Self {
            digest_key,
            state: tokio::sync::Mutex::new(AuthAbuseState {
                ip_windows: HashMap::with_capacity(max_ip_records),
                identity_windows: HashMap::with_capacity(max_identity_records),
            }),
            max_ip_records,
            max_identity_records,
        }
    }

    /// Admit one source-IP attempt. IPv6 sources share a `/64` budget.
    pub async fn check_ip(
        &self,
        endpoint: AuthEndpoint,
        ip: IpAddr,
    ) -> Result<(), AuthThrottleRejection> {
        self.check_ip_at(endpoint, ip, Instant::now()).await
    }

    pub(super) async fn check_ip_at(
        &self,
        endpoint: AuthEndpoint,
        ip: IpAddr,
        now: Instant,
    ) -> Result<(), AuthThrottleRejection> {
        let digest = self.digest(b"ip-prefix", &canonical_ip_prefix(ip));
        let mut state = self.state.lock().await;
        apply_limits(
            &mut state.ip_windows,
            self.max_ip_records,
            endpoint,
            AuthThrottleDimension::IpPrefix,
            digest,
            ip_limits(endpoint),
            now,
        )
    }

    /// Admit one normalized identity or opaque-token attempt.
    pub async fn check_identity(
        &self,
        endpoint: AuthEndpoint,
        identity: AuthIdentity<'_>,
    ) -> Result<(), AuthThrottleRejection> {
        let (dimension, digest) = match identity {
            AuthIdentity::Email(value) => {
                let normalized = Zeroizing::new(value.trim().to_lowercase());
                (
                    AuthThrottleDimension::Email,
                    self.digest(b"email", normalized.as_bytes()),
                )
            }
            AuthIdentity::UserName(value) => {
                let normalized = Zeroizing::new(value.trim().to_lowercase());
                (
                    AuthThrottleDimension::UserName,
                    self.digest(b"user-name", normalized.as_bytes()),
                )
            }
            AuthIdentity::Token(value) => (
                AuthThrottleDimension::Token,
                self.digest(b"token", value),
            ),
        };
        let limits = identity_limits(endpoint, dimension);
        if limits.is_empty() {
            return Err(AuthThrottleRejection::new(
                endpoint,
                dimension,
                CAPACITY_RETRY_AFTER,
                true,
            ));
        }

        self.check_identity_digest_at(endpoint, dimension, digest, limits, Instant::now())
            .await
    }

    async fn check_identity_digest_at(
        &self,
        endpoint: AuthEndpoint,
        dimension: AuthThrottleDimension,
        digest: AuthKeyDigest,
        limits: &[FixedWindowLimit],
        now: Instant,
    ) -> Result<(), AuthThrottleRejection> {
        let mut state = self.state.lock().await;
        apply_limits(
            &mut state.identity_windows,
            self.max_identity_records,
            endpoint,
            dimension,
            digest,
            limits,
            now,
        )
    }

    /// Remove expired fixed-window records without admitting new identities.
    pub async fn prune_expired(&self) -> AuthAbusePruneReport {
        self.prune_expired_at(Instant::now()).await
    }

    pub(super) async fn prune_expired_at(&self, now: Instant) -> AuthAbusePruneReport {
        let mut state = self.state.lock().await;
        let ip_before = state.ip_windows.len();
        let identity_before = state.identity_windows.len();
        state.ip_windows.retain(|_, window| window.expires_at > now);
        state
            .identity_windows
            .retain(|_, window| window.expires_at > now);
        AuthAbusePruneReport {
            ip_records_removed: ip_before.saturating_sub(state.ip_windows.len()),
            identity_records_removed: identity_before.saturating_sub(state.identity_windows.len()),
            ip_records_retained: state.ip_windows.len(),
            identity_records_retained: state.identity_windows.len(),
        }
    }

    fn digest(&self, domain: &[u8], value: &[u8]) -> AuthKeyDigest {
        let mut digest = Sha256::new();
        digest.update(self.digest_key.as_ref());
        digest.update((domain.len() as u64).to_be_bytes());
        digest.update(domain);
        digest.update((value.len() as u64).to_be_bytes());
        digest.update(value);
        AuthKeyDigest(digest.finalize().into())
    }
}

#[derive(Default)]
struct AuthAbuseState {
    ip_windows: HashMap<WindowKey, FixedWindow>,
    identity_windows: HashMap<WindowKey, FixedWindow>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct AuthKeyDigest([u8; AUTH_KEY_BYTES]);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct WindowKey {
    endpoint: AuthEndpoint,
    ordinal: u8,
    digest: AuthKeyDigest,
}

#[derive(Clone, Copy, Debug)]
struct FixedWindow {
    attempts: u32,
    expires_at: Instant,
}

fn apply_limits(
    windows: &mut HashMap<WindowKey, FixedWindow>,
    capacity: usize,
    endpoint: AuthEndpoint,
    dimension: AuthThrottleDimension,
    digest: AuthKeyDigest,
    limits: &[FixedWindowLimit],
    now: Instant,
) -> Result<(), AuthThrottleRejection> {
    let missing = limits
        .iter()
        .enumerate()
        .filter(|(ordinal, _)| {
            let key = WindowKey {
                endpoint,
                ordinal: *ordinal as u8,
                digest,
            };
            !windows.contains_key(&key)
        })
        .count();
    if windows.len().saturating_add(missing) > capacity {
        return Err(AuthThrottleRejection::new(
            endpoint,
            dimension,
            CAPACITY_RETRY_AFTER,
            true,
        ));
    }

    let mut retry_after = Duration::ZERO;
    for (ordinal, limit) in limits.iter().enumerate() {
        let key = WindowKey {
            endpoint,
            ordinal: ordinal as u8,
            digest,
        };
        let window = windows.entry(key).or_insert(FixedWindow {
            attempts: 0,
            expires_at: now + limit.duration,
        });
        if window.expires_at <= now {
            *window = FixedWindow {
                attempts: 0,
                expires_at: now + limit.duration,
            };
        }
        if window.attempts >= limit.attempts {
            retry_after = retry_after.max(window.expires_at.saturating_duration_since(now));
        } else {
            window.attempts += 1;
        }
    }

    if retry_after.is_zero() {
        Ok(())
    } else {
        Err(AuthThrottleRejection::new(
            endpoint,
            dimension,
            retry_after,
            false,
        ))
    }
}

fn canonical_ip_prefix(ip: IpAddr) -> [u8; 16] {
    match ip {
        IpAddr::V4(ip) => ip.to_ipv6_mapped().octets(),
        IpAddr::V6(ip) => {
            let bits = u128::from(ip) & (u128::MAX << 64);
            Ipv6Addr::from(bits).octets()
        }
    }
}
