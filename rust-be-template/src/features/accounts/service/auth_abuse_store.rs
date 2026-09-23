//! One fixed-capacity fixed-window table for a single endpoint and throttle dimension.

use std::{
    collections::{BTreeMap, HashMap},
    time::{Duration, Instant},
};

use crate::features::accounts::service::auth_abuse_policy::{
    FixedWindowLimit, MAX_WINDOWS_PER_POLICY,
};

/// Keyed digest of one throttled identity; raw identities never reach this table.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct AuthKeyDigest(pub(super) [u8; 32]);

#[derive(Clone, Copy, Debug)]
struct FixedWindow {
    attempts: u32,
    expires_at: Instant,
}

struct WindowRecord {
    windows: [FixedWindow; MAX_WINDOWS_PER_POLICY],
    /// Present while the record holds at most one attempt in every live window.
    evictable_sequence: Option<u64>,
}

/// Why a counted attempt could not be recorded or admitted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum StoreRejection {
    /// A window is exhausted until the contained delay elapses.
    Exhausted(Duration),
    /// The table is full of records that each carry more than one attempt.
    Saturated,
}

/// Outcome of recording one attempt or failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Recorded {
    /// Whether any window reached its limit with this attempt.
    pub(super) exhausted: bool,
}

/// Bounded fixed-window counters with oldest-single-attempt eviction.
///
/// When the table is full, the oldest record holding a single attempt is evicted instead of
/// rejecting every new key. A spray of one-off keys therefore displaces only other one-off
/// keys, whose loss forgives at most one attempt, while records that are close to a limit are
/// retained so an attacker cannot reset their own budget by flooding the table.
pub(super) struct WindowStore {
    records: HashMap<AuthKeyDigest, WindowRecord>,
    evictable: BTreeMap<u64, AuthKeyDigest>,
    next_sequence: u64,
    capacity: usize,
}

impl WindowStore {
    pub(super) fn new(capacity: usize) -> Self {
        Self {
            records: HashMap::with_capacity(capacity.min(256)),
            evictable: BTreeMap::new(),
            next_sequence: 0,
            capacity,
        }
    }

    pub(super) fn len(&self) -> usize {
        self.records.len()
    }

    /// Checks every window without counting the request.
    pub(super) fn check(
        &self,
        digest: &AuthKeyDigest,
        limits: &[FixedWindowLimit],
        now: Instant,
    ) -> Result<(), StoreRejection> {
        let record = match self.records.get(digest) {
            Some(record) => record,
            None => return Ok(()),
        };
        let mut retry_after = Duration::ZERO;
        for (window, limit) in record.windows.iter().zip(limits) {
            if window.expires_at > now && window.attempts >= limit.attempts {
                retry_after = retry_after.max(window.expires_at.saturating_duration_since(now));
            }
        }
        if retry_after.is_zero() {
            Ok(())
        } else {
            Err(StoreRejection::Exhausted(retry_after))
        }
    }

    /// Checks every window and counts the request only when all of them admit it.
    ///
    /// A request rejected by a short window is not charged to longer windows, so a burst that
    /// trips the per-minute limit does not also drain the hourly budget.
    pub(super) fn admit(
        &mut self,
        digest: AuthKeyDigest,
        limits: &[FixedWindowLimit],
        now: Instant,
    ) -> Result<Recorded, StoreRejection> {
        self.check(&digest, limits, now)?;
        self.record(digest, limits, now)
    }

    /// Counts one attempt or failure in every window, creating the record when needed.
    pub(super) fn record(
        &mut self,
        digest: AuthKeyDigest,
        limits: &[FixedWindowLimit],
        now: Instant,
    ) -> Result<Recorded, StoreRejection> {
        if !self.records.contains_key(&digest) {
            self.make_room()?;
            self.records.insert(
                digest,
                WindowRecord {
                    windows: [FixedWindow {
                        attempts: 0,
                        expires_at: now,
                    }; MAX_WINDOWS_PER_POLICY],
                    evictable_sequence: None,
                },
            );
        }
        let record = match self.records.get_mut(&digest) {
            Some(record) => record,
            None => return Err(StoreRejection::Saturated),
        };
        let mut exhausted = false;
        let mut max_attempts = 0_u32;
        for (window, limit) in record.windows.iter_mut().zip(limits) {
            if window.expires_at <= now {
                *window = FixedWindow {
                    attempts: 0,
                    expires_at: now + limit.duration,
                };
            }
            window.attempts = window.attempts.saturating_add(1);
            exhausted |= window.attempts >= limit.attempts;
            max_attempts = max_attempts.max(window.attempts);
        }
        if let Some(sequence) = record.evictable_sequence.take() {
            self.evictable.remove(&sequence);
        }
        if max_attempts <= 1 {
            let sequence = self.next_sequence;
            self.next_sequence = self.next_sequence.wrapping_add(1);
            record.evictable_sequence = Some(sequence);
            self.evictable.insert(sequence, digest);
        }
        Ok(Recorded { exhausted })
    }

    /// Removes a record, for example after a successful login clears its failure count.
    pub(super) fn forget(&mut self, digest: &AuthKeyDigest) {
        if let Some(record) = self.records.remove(digest)
            && let Some(sequence) = record.evictable_sequence
        {
            self.evictable.remove(&sequence);
        }
    }

    /// Drops records whose windows have all expired; returns the number removed.
    pub(super) fn prune(&mut self, now: Instant) -> usize {
        let before = self.records.len();
        let evictable = &mut self.evictable;
        self.records.retain(|_, record| {
            let live = record.windows.iter().any(|window| window.expires_at > now);
            if !live && let Some(sequence) = record.evictable_sequence {
                evictable.remove(&sequence);
            }
            live
        });
        before.saturating_sub(self.records.len())
    }

    fn make_room(&mut self) -> Result<(), StoreRejection> {
        if self.records.len() < self.capacity {
            return Ok(());
        }
        match self.evictable.pop_first() {
            Some((_, digest)) => {
                self.records.remove(&digest);
                Ok(())
            }
            None => Err(StoreRejection::Saturated),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIMITS: [FixedWindowLimit; 2] = [
        FixedWindowLimit {
            attempts: 2,
            duration: Duration::from_secs(60),
        },
        FixedWindowLimit {
            attempts: 3,
            duration: Duration::from_secs(3_600),
        },
    ];

    fn digest(byte: u8) -> AuthKeyDigest {
        AuthKeyDigest([byte; 32])
    }

    #[test]
    fn short_window_rejections_do_not_drain_the_long_window() {
        let mut store = WindowStore::new(4);
        let start = Instant::now();
        assert!(store.admit(digest(1), &LIMITS, start).is_ok());
        assert!(store.admit(digest(1), &LIMITS, start).is_ok());
        for _ in 0..5 {
            assert!(matches!(
                store.admit(digest(1), &LIMITS, start),
                Err(StoreRejection::Exhausted(_))
            ));
        }
        let next_minute = start + Duration::from_secs(61);
        assert!(store.admit(digest(1), &LIMITS, next_minute).is_ok());
        assert!(matches!(
            store.admit(digest(1), &LIMITS, next_minute),
            Err(StoreRejection::Exhausted(_))
        ));
    }

    #[test]
    fn full_table_evicts_the_oldest_single_attempt_record() {
        let mut store = WindowStore::new(2);
        let now = Instant::now();
        assert!(store.admit(digest(1), &LIMITS, now).is_ok());
        assert!(store.admit(digest(1), &LIMITS, now).is_ok());
        assert!(store.admit(digest(2), &LIMITS, now).is_ok());
        assert!(store.admit(digest(3), &LIMITS, now).is_ok());
        assert_eq!(store.len(), 2);
        // The multi-attempt record survived, so its exhausted window still rejects.
        assert!(matches!(
            store.admit(digest(1), &LIMITS, now),
            Err(StoreRejection::Exhausted(_))
        ));
        assert!(!store.records.contains_key(&digest(2)));
    }

    #[test]
    fn table_of_multi_attempt_records_fails_closed() {
        let mut store = WindowStore::new(1);
        let now = Instant::now();
        assert!(store.record(digest(1), &LIMITS, now).is_ok());
        assert!(store.record(digest(1), &LIMITS, now).is_ok());
        assert_eq!(
            store.record(digest(2), &LIMITS, now),
            Err(StoreRejection::Saturated)
        );
    }

    #[test]
    fn check_counts_nothing_and_record_reports_exhaustion() {
        let mut store = WindowStore::new(2);
        let now = Instant::now();
        assert!(store.check(&digest(1), &LIMITS, now).is_ok());
        assert_eq!(store.len(), 0);
        assert_eq!(
            store.record(digest(1), &LIMITS, now),
            Ok(Recorded { exhausted: false })
        );
        assert_eq!(
            store.record(digest(1), &LIMITS, now),
            Ok(Recorded { exhausted: true })
        );
        store.forget(&digest(1));
        assert!(store.check(&digest(1), &LIMITS, now).is_ok());
        assert!(store.evictable.is_empty());
    }

    #[test]
    fn prune_removes_fully_expired_records_and_their_eviction_entries() {
        let mut store = WindowStore::new(4);
        let now = Instant::now();
        assert!(store.admit(digest(1), &LIMITS, now).is_ok());
        assert_eq!(store.prune(now + Duration::from_secs(120)), 0);
        assert_eq!(store.prune(now + Duration::from_secs(3_601)), 1);
        assert!(store.evictable.is_empty());
    }
}
