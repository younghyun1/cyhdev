//! Fixed-capacity per-account write budgets for authenticated content writes.
//!
//! Each account holds one fixed window per write kind. The actor table has a
//! hard capacity: a novel account is admitted only while a slot is free, and
//! expired actors are purged only when admission needs space. At capacity the
//! limiter fails closed with a retry hint instead of growing, so the process
//! never retains more than `max_actors` entries.

use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

use scc::hash_map::Entry;
use uuid::Uuid;

/// Retry hint returned when the actor table is saturated.
const SATURATED_RETRY_AFTER: Duration = Duration::from_secs(60);

/// Maximum attempts admitted per fixed window for one write kind.
#[derive(Clone, Copy, Debug)]
pub struct WriteBudget {
    pub attempts: u32,
    pub window: Duration,
}

/// Rejection carrying the `Retry-After` hint and whether capacity caused it.
#[derive(Clone, Copy, Debug)]
pub struct WriteRejection {
    pub retry_after: Duration,
    pub saturated: bool,
}

#[derive(Clone, Copy)]
struct Window {
    attempts: u32,
    expires_at: Instant,
}

/// Per-account limiter with `KINDS` independent budgets.
pub struct ActorWriteLimiter<const KINDS: usize> {
    actors: scc::HashMap<Uuid, [Window; KINDS]>,
    active_slots: AtomicUsize,
    max_actors: usize,
    budgets: [WriteBudget; KINDS],
}

impl<const KINDS: usize> ActorWriteLimiter<KINDS> {
    pub fn new(max_actors: usize, budgets: [WriteBudget; KINDS]) -> Self {
        Self {
            actors: scc::HashMap::with_capacity(max_actors),
            active_slots: AtomicUsize::new(0),
            max_actors,
            budgets,
        }
    }

    /// Charges one attempt of budget `kind` for `user_id`.
    ///
    /// An out-of-range `kind` is a caller bug; it fails closed as saturation
    /// rather than panicking or admitting an unmetered write.
    pub async fn check(&self, user_id: Uuid, kind: usize) -> Result<(), WriteRejection> {
        self.check_at(user_id, kind, Instant::now()).await
    }

    pub(crate) async fn check_at(
        &self,
        user_id: Uuid,
        kind: usize,
        now: Instant,
    ) -> Result<(), WriteRejection> {
        let Some(budget) = self.budgets.get(kind).copied() else {
            return Err(saturated());
        };
        match self.actors.entry_async(user_id).await {
            Entry::Occupied(mut entry) => return charge(entry.get_mut(), kind, budget, now),
            Entry::Vacant(entry) => drop(entry),
        }
        if !self.try_reserve() {
            self.purge_expired(now).await;
            if !self.try_reserve() {
                return Err(saturated());
            }
        }
        match self.actors.entry_async(user_id).await {
            Entry::Vacant(entry) => {
                let mut windows = self.fresh_windows(now);
                let result = charge(&mut windows, kind, budget, now);
                entry.insert_entry(windows);
                result
            }
            Entry::Occupied(mut entry) => {
                // Another request admitted this actor between the two lookups.
                self.release();
                charge(entry.get_mut(), kind, budget, now)
            }
        }
    }

    fn fresh_windows(&self, now: Instant) -> [Window; KINDS] {
        std::array::from_fn(|index| Window {
            attempts: 0,
            expires_at: now + self.budgets[index].window,
        })
    }

    async fn purge_expired(&self, now: Instant) {
        self.actors
            .iter_mut_async(|entry| {
                if entry.iter().all(|window| window.expires_at <= now) {
                    let _ = entry.consume();
                    self.release();
                }
                true
            })
            .await;
    }

    fn try_reserve(&self) -> bool {
        self.active_slots
            .try_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                (current < self.max_actors).then_some(current + 1)
            })
            .is_ok()
    }

    fn release(&self) {
        self.active_slots.fetch_sub(1, Ordering::AcqRel);
    }
}

fn saturated() -> WriteRejection {
    WriteRejection {
        retry_after: SATURATED_RETRY_AFTER,
        saturated: true,
    }
}

fn charge<const KINDS: usize>(
    windows: &mut [Window; KINDS],
    kind: usize,
    budget: WriteBudget,
    now: Instant,
) -> Result<(), WriteRejection> {
    let Some(window) = windows.get_mut(kind) else {
        return Err(saturated());
    };
    if window.expires_at <= now {
        *window = Window {
            attempts: 0,
            expires_at: now + budget.window,
        };
    }
    if window.attempts >= budget.attempts {
        return Err(WriteRejection {
            retry_after: window
                .expires_at
                .saturating_duration_since(now)
                .max(Duration::from_secs(1)),
            saturated: false,
        });
    }
    window.attempts += 1;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const WINDOW: Duration = Duration::from_secs(600);

    fn limiter(max_actors: usize) -> ActorWriteLimiter<2> {
        ActorWriteLimiter::new(
            max_actors,
            [
                WriteBudget {
                    attempts: 2,
                    window: WINDOW,
                },
                WriteBudget {
                    attempts: 3,
                    window: WINDOW,
                },
            ],
        )
    }

    #[tokio::test]
    async fn budgets_are_independent_per_kind() {
        let limiter = limiter(2);
        let user = Uuid::new_v4();
        let now = Instant::now();
        for _ in 0..2 {
            assert!(limiter.check_at(user, 0, now).await.is_ok());
        }
        let exhausted = limiter.check_at(user, 0, now).await;
        assert!(matches!(exhausted, Err(rejection) if !rejection.saturated));
        assert!(limiter.check_at(user, 1, now).await.is_ok());
    }

    #[tokio::test]
    async fn window_expiry_restores_the_budget() {
        let limiter = limiter(1);
        let user = Uuid::new_v4();
        let now = Instant::now();
        for _ in 0..2 {
            assert!(limiter.check_at(user, 0, now).await.is_ok());
        }
        assert!(limiter.check_at(user, 0, now).await.is_err());
        let later = now + WINDOW + Duration::from_secs(1);
        assert!(limiter.check_at(user, 0, later).await.is_ok());
    }

    #[tokio::test]
    async fn capacity_fails_closed_until_expiry() {
        let limiter = limiter(1);
        let now = Instant::now();
        assert!(limiter.check_at(Uuid::new_v4(), 0, now).await.is_ok());
        let rejection = limiter.check_at(Uuid::new_v4(), 1, now).await;
        assert!(matches!(rejection, Err(rejection) if rejection.saturated));
        let later = now + WINDOW + Duration::from_secs(1);
        assert!(limiter.check_at(Uuid::new_v4(), 1, later).await.is_ok());
    }

    #[tokio::test]
    async fn unknown_kind_fails_closed() {
        let limiter = limiter(4);
        let rejection = limiter.check_at(Uuid::new_v4(), 2, Instant::now()).await;
        assert!(matches!(rejection, Err(rejection) if rejection.saturated));
    }
}
