//! Fixed-capacity per-account forum write budgets.

use scc::hash_map::Entry;
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};
use uuid::Uuid;

pub const MAX_FORUM_WRITE_ACTORS: usize = 16_384;
const WRITE_WINDOW: Duration = Duration::from_secs(10 * 60);
const TOPIC_ATTEMPTS: u32 = 3;
const REPLY_ATTEMPTS: u32 = 30;

#[derive(Clone, Copy)]
pub(super) enum ForumWriteKind {
    Topic,
    Reply,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ForumWriteRejection {
    pub(super) retry_after: Duration,
    pub(super) saturated: bool,
}

pub struct ForumWriteLimiter {
    actors: scc::HashMap<Uuid, ActorWindows>,
    active_slots: AtomicUsize,
    max_actors: usize,
}

impl ForumWriteLimiter {
    pub fn new() -> Self {
        Self::with_capacity(MAX_FORUM_WRITE_ACTORS)
    }

    fn with_capacity(max_actors: usize) -> Self {
        Self {
            actors: scc::HashMap::with_capacity(max_actors),
            active_slots: AtomicUsize::new(0),
            max_actors,
        }
    }

    pub(super) async fn check(
        &self,
        user_id: Uuid,
        kind: ForumWriteKind,
    ) -> Result<(), ForumWriteRejection> {
        self.check_at(user_id, kind, Instant::now()).await
    }

    async fn check_at(
        &self,
        user_id: Uuid,
        kind: ForumWriteKind,
        now: Instant,
    ) -> Result<(), ForumWriteRejection> {
        match self.actors.entry_async(user_id).await {
            Entry::Occupied(mut entry) => return charge(entry.get_mut(), kind, now),
            Entry::Vacant(entry) => drop(entry),
        }
        if !self.try_reserve() {
            self.purge_expired(now).await;
            if !self.try_reserve() {
                return Err(ForumWriteRejection {
                    retry_after: Duration::from_secs(60),
                    saturated: true,
                });
            }
        }
        match self.actors.entry_async(user_id).await {
            Entry::Vacant(entry) => {
                let mut windows = ActorWindows::new(now);
                let result = charge(&mut windows, kind, now);
                entry.insert_entry(windows);
                result
            }
            Entry::Occupied(mut entry) => {
                self.release();
                charge(entry.get_mut(), kind, now)
            }
        }
    }

    async fn purge_expired(&self, now: Instant) {
        self.actors
            .iter_mut_async(|entry| {
                if entry.topic.expires_at <= now && entry.reply.expires_at <= now {
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

impl Default for ForumWriteLimiter {
    fn default() -> Self {
        Self::new()
    }
}

struct ActorWindows {
    topic: Window,
    reply: Window,
}
impl ActorWindows {
    fn new(now: Instant) -> Self {
        Self {
            topic: Window::new(now),
            reply: Window::new(now),
        }
    }
}

struct Window {
    attempts: u32,
    expires_at: Instant,
}
impl Window {
    fn new(now: Instant) -> Self {
        Self {
            attempts: 0,
            expires_at: now + WRITE_WINDOW,
        }
    }
}

fn charge(
    windows: &mut ActorWindows,
    kind: ForumWriteKind,
    now: Instant,
) -> Result<(), ForumWriteRejection> {
    let (window, maximum) = match kind {
        ForumWriteKind::Topic => (&mut windows.topic, TOPIC_ATTEMPTS),
        ForumWriteKind::Reply => (&mut windows.reply, REPLY_ATTEMPTS),
    };
    if window.expires_at <= now {
        *window = Window::new(now);
    }
    if window.attempts >= maximum {
        Err(ForumWriteRejection {
            retry_after: window
                .expires_at
                .saturating_duration_since(now)
                .max(Duration::from_secs(1)),
            saturated: false,
        })
    } else {
        window.attempts += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn topic_and_reply_budgets_are_independent() {
        let limiter = ForumWriteLimiter::with_capacity(2);
        let user = Uuid::new_v4();
        let now = Instant::now();
        for _ in 0..TOPIC_ATTEMPTS {
            assert!(
                limiter
                    .check_at(user, ForumWriteKind::Topic, now)
                    .await
                    .is_ok()
            );
        }
        assert!(
            limiter
                .check_at(user, ForumWriteKind::Topic, now)
                .await
                .is_err()
        );
        assert!(
            limiter
                .check_at(user, ForumWriteKind::Reply, now)
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn capacity_fails_closed_until_expiry() {
        let limiter = ForumWriteLimiter::with_capacity(1);
        let now = Instant::now();
        assert!(
            limiter
                .check_at(Uuid::new_v4(), ForumWriteKind::Topic, now)
                .await
                .is_ok()
        );
        let rejection = limiter
            .check_at(Uuid::new_v4(), ForumWriteKind::Reply, now)
            .await;
        assert!(matches!(
            rejection,
            Err(ForumWriteRejection {
                saturated: true,
                ..
            })
        ));
        let later = now + WRITE_WINDOW + Duration::from_secs(1);
        assert!(
            limiter
                .check_at(Uuid::new_v4(), ForumWriteKind::Reply, later)
                .await
                .is_ok()
        );
    }
}
