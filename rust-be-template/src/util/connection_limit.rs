//! Bounded admission for long-lived connections, globally and per client network.
//!
//! A global cap alone would let one client take every slot, so each admission also
//! counts against the client's network: the exact address for IPv4 and the /64
//! prefix for IPv6, since one subscriber typically controls an entire /64. The
//! per-client table holds only networks with a live permit, so it can never exceed
//! the global cap.

use std::{
    net::{IpAddr, Ipv6Addr},
    sync::{
        Arc,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    },
};

use scc::hash_map::Entry;

/// Why an admission was refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectionRejection {
    GlobalLimit,
    ClientLimit,
}

/// Shared admission state; clones refer to the same counters.
#[derive(Clone)]
pub struct ConnectionLimiter {
    inner: Arc<LimiterState>,
}

struct LimiterState {
    name: &'static str,
    max_total: usize,
    max_per_client: usize,
    active: AtomicUsize,
    per_client: scc::HashMap<IpAddr, usize>,
    rejections: AtomicU64,
}

/// Holds one admitted slot until dropped.
pub struct ConnectionPermit {
    state: Arc<LimiterState>,
    client: IpAddr,
}

impl ConnectionLimiter {
    /// `name` labels rejection logs; both limits are clamped to at least one.
    pub fn new(name: &'static str, max_total: usize, max_per_client: usize) -> Self {
        Self {
            inner: Arc::new(LimiterState {
                name,
                max_total: max_total.max(1),
                max_per_client: max_per_client.max(1),
                active: AtomicUsize::new(0),
                per_client: scc::HashMap::new(),
                rejections: AtomicU64::new(0),
            }),
        }
    }

    /// Admits one connection for `ip` or reports which bound refused it.
    pub fn try_acquire(&self, ip: IpAddr) -> Result<ConnectionPermit, ConnectionRejection> {
        let state = &self.inner;
        let reserved = state
            .active
            .try_update(Ordering::AcqRel, Ordering::Acquire, |active| {
                (active < state.max_total).then_some(active + 1)
            })
            .is_ok();
        if !reserved {
            return Err(self.reject(ConnectionRejection::GlobalLimit));
        }
        let client = client_network(ip);
        let admitted = match state.per_client.entry_sync(client) {
            Entry::Occupied(mut occupied) => {
                let count = occupied.get_mut();
                if *count < state.max_per_client {
                    *count += 1;
                    true
                } else {
                    false
                }
            }
            Entry::Vacant(vacant) => {
                vacant.insert_entry(1);
                true
            }
        };
        if admitted {
            Ok(ConnectionPermit {
                state: Arc::clone(&self.inner),
                client,
            })
        } else {
            state.active.fetch_sub(1, Ordering::AcqRel);
            Err(self.reject(ConnectionRejection::ClientLimit))
        }
    }

    /// Currently admitted connections.
    pub fn active(&self) -> usize {
        self.inner.active.load(Ordering::Acquire)
    }

    /// Logs rejection bursts at powers of two so a flood cannot flood the logs too.
    fn reject(&self, rejection: ConnectionRejection) -> ConnectionRejection {
        let total = self.inner.rejections.fetch_add(1, Ordering::Relaxed) + 1;
        if total.is_power_of_two() {
            tracing::warn!(
                limiter = self.inner.name,
                reason = ?rejection,
                rejected_total = total,
                max_total = self.inner.max_total,
                max_per_client = self.inner.max_per_client,
                "Rejected connection admission"
            );
        }
        rejection
    }
}

impl Drop for ConnectionPermit {
    fn drop(&mut self) {
        let _ = self.state.per_client.remove_if_sync(&self.client, |count| {
            *count = count.saturating_sub(1);
            *count == 0
        });
        self.state.active.fetch_sub(1, Ordering::AcqRel);
    }
}

/// Groups addresses by the network one client plausibly controls.
pub fn client_network(ip: IpAddr) -> IpAddr {
    match ip.to_canonical() {
        IpAddr::V4(address) => IpAddr::V4(address),
        IpAddr::V6(address) => {
            let prefix = u128::from(address) & (u128::MAX << 64);
            IpAddr::V6(Ipv6Addr::from(prefix))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{error::Error, net::IpAddr};

    use super::{ConnectionLimiter, ConnectionRejection, client_network};

    type TestResult = Result<(), Box<dyn Error>>;

    fn ip(value: &str) -> Result<IpAddr, Box<dyn Error>> {
        Ok(value.parse::<IpAddr>()?)
    }

    #[test]
    fn per_client_limit_applies_before_the_global_limit() -> TestResult {
        let limiter = ConnectionLimiter::new("test", 3, 2);
        let first = limiter.try_acquire(ip("192.0.2.1")?);
        let second = limiter.try_acquire(ip("192.0.2.1")?);
        assert!(first.is_ok() && second.is_ok());
        assert_eq!(
            limiter.try_acquire(ip("192.0.2.1")?).err(),
            Some(ConnectionRejection::ClientLimit)
        );
        let other = limiter.try_acquire(ip("198.51.100.7")?);
        assert!(other.is_ok());
        assert_eq!(
            limiter.try_acquire(ip("203.0.113.9")?).err(),
            Some(ConnectionRejection::GlobalLimit)
        );
        assert_eq!(limiter.active(), 3);
        drop(first);
        assert_eq!(limiter.active(), 2);
        assert!(limiter.try_acquire(ip("192.0.2.1")?).is_ok());
        Ok(())
    }

    #[test]
    fn released_permits_remove_idle_client_entries() -> TestResult {
        let limiter = ConnectionLimiter::new("test", 8, 1);
        let permit = limiter.try_acquire(ip("192.0.2.1")?);
        assert!(permit.is_ok());
        drop(permit);
        assert_eq!(limiter.active(), 0);
        assert_eq!(limiter.inner.per_client.len(), 0);
        assert!(limiter.try_acquire(ip("192.0.2.1")?).is_ok());
        Ok(())
    }

    #[test]
    fn ipv6_clients_share_a_slash_64_and_mapped_ipv4_is_canonical() -> TestResult {
        assert_eq!(
            client_network(ip("2001:db8:1:2:aaaa::1")?),
            ip("2001:db8:1:2::")?
        );
        assert_eq!(client_network(ip("::ffff:192.0.2.1")?), ip("192.0.2.1")?);
        let limiter = ConnectionLimiter::new("test", 8, 1);
        let held = limiter.try_acquire(ip("2001:db8:1:2::1")?);
        assert!(held.is_ok());
        assert_eq!(
            limiter.try_acquire(ip("2001:db8:1:2:ffff::9")?).err(),
            Some(ConnectionRejection::ClientLimit)
        );
        assert!(limiter.try_acquire(ip("2001:db8:1:3::1")?).is_ok());
        Ok(())
    }
}
