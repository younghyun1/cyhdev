//! Bounded per-client HTTP request budgets with the historical global rate policy.
//!
//! The default replenishes every 63 milliseconds, with a burst of 1,024. Only
//! fully replenished clients may leave the table, so filling it cannot reset an
//! attacker's debt. IPv6 addresses share their subscriber's /64 budget.

use std::{
    net::IpAddr,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

use scc::hash_map::Entry;
use tokio::sync::Mutex;

use super::connection_limit::client_network;

/// Historical interval between replenished request permits.
pub const REQUEST_REFILL_INTERVAL: Duration = Duration::from_millis(63);
/// Historical maximum instantaneous request burst per client network.
pub const REQUEST_BURST_SIZE: u32 = 1_024;
/// Maximum retained client networks, including reservations awaiting insertion.
pub const MAX_REQUEST_RATE_CLIENTS: usize = 16_384;
const SWEEP_INTERVAL: Duration = Duration::from_secs(1);

/// Finite replenishment policy; retained client state is independent of its quota.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RequestRatePolicy {
    refill_interval: Duration,
    burst_size: u32,
}

impl RequestRatePolicy {
    /// Public service policy, unchanged by optimization fixture configuration.
    pub const HISTORICAL: Self = Self {
        refill_interval: REQUEST_REFILL_INTERVAL,
        burst_size: REQUEST_BURST_SIZE,
    };

    /// Bounds alternate policies without allowing zero-cost or unlimited requests.
    pub(crate) fn bounded(refill_interval: Duration, burst_size: u32) -> anyhow::Result<Self> {
        if !(Duration::from_micros(1)..=REQUEST_REFILL_INTERVAL).contains(&refill_interval)
            || !(1..=16_384).contains(&burst_size)
        {
            return Err(anyhow::anyhow!(
                "request admission policy exceeds its finite bounds"
            ));
        }
        Ok(Self {
            refill_interval,
            burst_size,
        })
    }

    /// Replenishment interval reported in sanitized startup diagnostics.
    pub(crate) const fn refill_interval(self) -> Duration {
        self.refill_interval
    }

    /// Finite burst reported in sanitized startup diagnostics.
    pub(crate) const fn burst_size(self) -> u32 {
        self.burst_size
    }
}

/// Retry hint and whether a novel client was rejected by table capacity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RequestRateRejection {
    pub retry_after: Duration,
    pub saturated: bool,
}

/// Process-local request admission shared by every limited HTTP surface.
pub struct RequestRateLimiter {
    clients: scc::HashMap<IpAddr, Instant>,
    active_slots: AtomicUsize,
    max_clients: usize,
    next_sweep: Mutex<Option<Instant>>,
    policy: RequestRatePolicy,
}

impl RequestRateLimiter {
    /// Creates a limiter with the fixed production capacity and historical policy.
    pub fn new() -> Self {
        Self::with_policy(RequestRatePolicy::HISTORICAL)
    }

    /// Applies a startup-validated policy while preserving the fixed client-table cap.
    pub(crate) fn with_policy(policy: RequestRatePolicy) -> Self {
        Self::with_capacity(MAX_REQUEST_RATE_CLIENTS, policy)
    }

    fn with_capacity(max_clients: usize, policy: RequestRatePolicy) -> Self {
        Self {
            clients: scc::HashMap::with_capacity(max_clients),
            active_slots: AtomicUsize::new(0),
            max_clients,
            next_sweep: Mutex::new(None),
            policy,
        }
    }

    /// Charges one request to the canonical client address or IPv6 network.
    pub async fn check(&self, ip: IpAddr) -> Result<(), RequestRateRejection> {
        self.check_at(ip, Instant::now()).await
    }

    pub(crate) async fn check_at(
        &self,
        ip: IpAddr,
        now: Instant,
    ) -> Result<(), RequestRateRejection> {
        let client = client_network(ip);
        let mut swept = false;
        loop {
            match self.clients.entry_async(client).await {
                Entry::Occupied(mut entry) => return charge(entry.get_mut(), now, self.policy),
                Entry::Vacant(entry) => {
                    let reservation = match self.try_reserve() {
                        Some(reservation) => reservation,
                        None => {
                            // Release the entry lock before a scan touches its bucket.
                            drop(entry);
                            if swept {
                                return Err(saturated());
                            }
                            self.purge_replenished(now).await;
                            swept = true;
                            continue;
                        }
                    };
                    let mut replenished_at = now;
                    let result = charge(&mut replenished_at, now, self.policy);
                    if result.is_ok() {
                        entry.insert_entry(replenished_at);
                        reservation.retain();
                    }
                    return result;
                }
            }
        }
    }

    async fn purge_replenished(&self, now: Instant) {
        let mut next_sweep = match self.next_sweep.try_lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };
        if next_sweep.is_some_and(|scheduled| scheduled > now) {
            return;
        }
        *next_sweep = now.checked_add(SWEEP_INTERVAL);
        // Keep the gate through the scan: concurrent saturation cannot start
        // another full-table walk or queue behind this cleanup.
        self.clients
            .iter_mut_async(|entry| {
                if *entry <= now {
                    let _ = entry.consume();
                    self.active_slots.fetch_sub(1, Ordering::AcqRel);
                }
                true
            })
            .await;
    }

    fn try_reserve(&self) -> Option<SlotReservation<'_>> {
        match self
            .active_slots
            .try_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                (current < self.max_clients).then_some(current + 1)
            }) {
            Ok(_) => Some(SlotReservation {
                active_slots: &self.active_slots,
                retained: false,
            }),
            Err(_) => None,
        }
    }
}

impl Default for RequestRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Releases a reserved slot unless a newly inserted table entry claims it.
struct SlotReservation<'a> {
    active_slots: &'a AtomicUsize,
    retained: bool,
}

impl SlotReservation<'_> {
    fn retain(mut self) {
        self.retained = true;
    }
}

impl Drop for SlotReservation<'_> {
    fn drop(&mut self) {
        if !self.retained {
            self.active_slots.fetch_sub(1, Ordering::AcqRel);
        }
    }
}

fn charge(
    replenished_at: &mut Instant,
    now: Instant,
    policy: RequestRatePolicy,
) -> Result<(), RequestRateRejection> {
    // GCRA retains only the time when all accumulated request debt is repaid.
    let tolerance = policy.refill_interval * (policy.burst_size - 1);
    let latest_admission = match now.checked_add(tolerance) {
        Some(latest_admission) => latest_admission,
        None => return Err(saturated()),
    };
    if *replenished_at > latest_admission {
        return Err(RequestRateRejection {
            retry_after: replenished_at.duration_since(latest_admission),
            saturated: false,
        });
    }
    match (*replenished_at)
        .max(now)
        .checked_add(policy.refill_interval)
    {
        Some(next) => {
            *replenished_at = next;
            Ok(())
        }
        None => Err(saturated()),
    }
}

fn saturated() -> RequestRateRejection {
    RequestRateRejection {
        retry_after: SWEEP_INTERVAL,
        saturated: true,
    }
}

#[cfg(test)]
#[path = "request_rate_limit_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "request_rate_policy_tests.rs"]
mod policy_tests;
