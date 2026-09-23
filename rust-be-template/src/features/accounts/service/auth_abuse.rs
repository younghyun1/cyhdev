//! Fixed-capacity process-local authentication throttles.

use std::{
    net::{IpAddr, Ipv6Addr},
    time::{Duration, Instant},
};

use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

use crate::features::accounts::domain::auth_abuse::{
    AuthAbusePruneReport, AuthEndpoint, AuthIdentity, AuthThrottleDimension, AuthThrottleRejection,
    FailureBudget,
};
use crate::features::accounts::service::{
    auth_abuse_policy::{THROTTLE_TABLES, ThrottleTable, table_index},
    auth_abuse_store::{AuthKeyDigest, StoreRejection, WindowStore},
};

const AUTH_KEY_BYTES: usize = 32;
const CAPACITY_RETRY_AFTER: Duration = Duration::from_secs(60);

/// Owns the bounded authentication-abuse authority for one backend process.
///
/// Every endpoint and dimension pair has its own table and lock, so saturation or contention
/// on one surface never affects another.
pub struct AuthAbuseService {
    digest_key: Zeroizing<[u8; AUTH_KEY_BYTES]>,
    tables: Vec<tokio::sync::Mutex<WindowStore>>,
}

impl AuthAbuseService {
    pub fn new() -> Result<Self, getrandom::Error> {
        let mut digest_key = Zeroizing::new([0_u8; AUTH_KEY_BYTES]);
        getrandom::fill(digest_key.as_mut())?;
        Ok(Self::with_capacity(digest_key, None))
    }

    /// Builds the limiter; `capacity_override` shrinks every table for tests.
    pub(super) fn with_capacity(
        digest_key: Zeroizing<[u8; AUTH_KEY_BYTES]>,
        capacity_override: Option<usize>,
    ) -> Self {
        let tables = THROTTLE_TABLES
            .iter()
            .map(|table| {
                tokio::sync::Mutex::new(WindowStore::new(
                    capacity_override.unwrap_or(table.capacity),
                ))
            })
            .collect();
        Self { digest_key, tables }
    }

    /// Admit and count one source-IP attempt. IPv6 sources share a `/64` budget.
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
        let digest = self.digest(b"ip-prefix", &[&canonical_ip_prefix(ip)]);
        self.admit(endpoint, AuthThrottleDimension::IpPrefix, digest, now)
            .await
    }

    /// Admit and count one normalized identity or opaque-token attempt.
    pub async fn check_identity(
        &self,
        endpoint: AuthEndpoint,
        identity: AuthIdentity<'_>,
    ) -> Result<(), AuthThrottleRejection> {
        let (dimension, digest) = self.identity_digest(identity);
        self.admit(endpoint, dimension, digest, Instant::now())
            .await
    }

    /// Rejects an identity whose failure budget is exhausted without counting this request.
    pub async fn ensure_failure_budget(
        &self,
        endpoint: AuthEndpoint,
        identity: AuthIdentity<'_>,
    ) -> Result<(), AuthThrottleRejection> {
        let (dimension, digest) = self.identity_digest(identity);
        let (index, table) = resolve_table(endpoint, dimension)?;
        let store = self.tables[index].lock().await;
        store
            .check(&digest, table.limits, Instant::now())
            .map_err(|rejection| map_rejection(endpoint, dimension, rejection))
    }

    /// Counts one failed attempt, such as a wrong password, against an identity budget.
    pub async fn record_failure(
        &self,
        endpoint: AuthEndpoint,
        identity: AuthIdentity<'_>,
    ) -> Result<FailureBudget, AuthThrottleRejection> {
        let (dimension, digest) = self.identity_digest(identity);
        self.record_failure_digest_at(endpoint, dimension, digest, Instant::now())
            .await
    }

    /// Clears an identity's counters, for example after the right password proves ownership.
    pub async fn forget_identity(&self, endpoint: AuthEndpoint, identity: AuthIdentity<'_>) {
        let (dimension, digest) = self.identity_digest(identity);
        if let Ok((index, _)) = resolve_table(endpoint, dimension) {
            self.tables[index].lock().await.forget(&digest);
        }
    }

    /// Remove expired fixed-window records without admitting new identities.
    pub async fn prune_expired(&self) -> AuthAbusePruneReport {
        self.prune_expired_at(Instant::now()).await
    }

    pub(super) async fn prune_expired_at(&self, now: Instant) -> AuthAbusePruneReport {
        let mut report = AuthAbusePruneReport::default();
        for (table, store) in THROTTLE_TABLES.iter().zip(&self.tables) {
            let mut store = store.lock().await;
            let removed = store.prune(now);
            let retained = store.len();
            drop(store);
            if table.dimension == AuthThrottleDimension::IpPrefix {
                report.ip_records_removed += removed;
                report.ip_records_retained += retained;
            } else {
                report.identity_records_removed += removed;
                report.identity_records_retained += retained;
            }
        }
        report
    }

    pub(super) async fn record_failure_digest_at(
        &self,
        endpoint: AuthEndpoint,
        dimension: AuthThrottleDimension,
        digest: AuthKeyDigest,
        now: Instant,
    ) -> Result<FailureBudget, AuthThrottleRejection> {
        let (index, table) = resolve_table(endpoint, dimension)?;
        let mut store = self.tables[index].lock().await;
        match store.record(digest, table.limits, now) {
            Ok(recorded) if recorded.exhausted => Ok(FailureBudget::Exhausted),
            Ok(_) => Ok(FailureBudget::Remaining),
            Err(rejection) => Err(map_rejection(endpoint, dimension, rejection)),
        }
    }

    async fn admit(
        &self,
        endpoint: AuthEndpoint,
        dimension: AuthThrottleDimension,
        digest: AuthKeyDigest,
        now: Instant,
    ) -> Result<(), AuthThrottleRejection> {
        let (index, table) = resolve_table(endpoint, dimension)?;
        let mut store = self.tables[index].lock().await;
        store
            .admit(digest, table.limits, now)
            .map(|_| ())
            .map_err(|rejection| map_rejection(endpoint, dimension, rejection))
    }

    pub(super) fn identity_digest(
        &self,
        identity: AuthIdentity<'_>,
    ) -> (AuthThrottleDimension, AuthKeyDigest) {
        match identity {
            AuthIdentity::Email(value) => {
                let normalized = Zeroizing::new(value.trim().to_lowercase());
                (
                    AuthThrottleDimension::Email,
                    self.digest(b"email", &[normalized.as_bytes()]),
                )
            }
            AuthIdentity::EmailFromIp(value, ip) => {
                let normalized = Zeroizing::new(value.trim().to_lowercase());
                (
                    AuthThrottleDimension::EmailAndIp,
                    self.digest(
                        b"email-ip",
                        &[normalized.as_bytes(), &canonical_ip_prefix(ip)],
                    ),
                )
            }
            AuthIdentity::UserName(value) => {
                let normalized = Zeroizing::new(value.trim().to_lowercase());
                (
                    AuthThrottleDimension::UserName,
                    self.digest(b"user-name", &[normalized.as_bytes()]),
                )
            }
            AuthIdentity::Token(value) => (
                AuthThrottleDimension::Token,
                self.digest(b"token", &[value]),
            ),
            AuthIdentity::Account(user_id) => (
                AuthThrottleDimension::Account,
                self.digest(b"account", &[user_id.as_bytes()]),
            ),
        }
    }

    /// Keyed, length-prefixed digest so distinct part splits can never collide.
    fn digest(&self, domain: &[u8], parts: &[&[u8]]) -> AuthKeyDigest {
        let mut digest = Sha256::new();
        digest.update(self.digest_key.as_ref());
        digest.update((domain.len() as u64).to_be_bytes());
        digest.update(domain);
        for part in parts {
            digest.update((part.len() as u64).to_be_bytes());
            digest.update(part);
        }
        AuthKeyDigest(digest.finalize().into())
    }
}

/// Finds the table for a pair; an unthrottled pair fails closed as a saturated rejection.
fn resolve_table(
    endpoint: AuthEndpoint,
    dimension: AuthThrottleDimension,
) -> Result<(usize, &'static ThrottleTable), AuthThrottleRejection> {
    match table_index(endpoint, dimension) {
        Some(index) => Ok((index, &THROTTLE_TABLES[index])),
        None => Err(AuthThrottleRejection::new(
            endpoint,
            dimension,
            CAPACITY_RETRY_AFTER,
            true,
        )),
    }
}

fn map_rejection(
    endpoint: AuthEndpoint,
    dimension: AuthThrottleDimension,
    rejection: StoreRejection,
) -> AuthThrottleRejection {
    match rejection {
        StoreRejection::Exhausted(retry_after) => {
            AuthThrottleRejection::new(endpoint, dimension, retry_after, false)
        }
        StoreRejection::Saturated => {
            AuthThrottleRejection::new(endpoint, dimension, CAPACITY_RETRY_AFTER, true)
        }
    }
}

/// IPv4 sources keep their full address and IPv6 sources share a `/64`. IPv4-mapped IPv6
/// addresses are treated as IPv4; masking them to `/64` would merge every IPv4 client.
pub(super) fn canonical_ip_prefix(ip: IpAddr) -> [u8; 16] {
    match ip {
        IpAddr::V4(ip) => ip.to_ipv6_mapped().octets(),
        IpAddr::V6(ip) if ip.to_ipv4_mapped().is_some() => ip.octets(),
        IpAddr::V6(ip) => {
            let bits = u128::from(ip) & (u128::MAX << 64);
            Ipv6Addr::from(bits).octets()
        }
    }
}
