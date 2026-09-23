//! Fixed-capacity per-account forum write budgets.

use std::time::Duration;

use uuid::Uuid;

use crate::util::actor_write_limiter::{ActorWriteLimiter, WriteBudget, WriteRejection};

pub const MAX_FORUM_WRITE_ACTORS: usize = 16_384;
const WRITE_WINDOW: Duration = Duration::from_secs(10 * 60);
const TOPIC_ATTEMPTS: u32 = 3;
const REPLY_ATTEMPTS: u32 = 30;

#[derive(Clone, Copy)]
pub(super) enum ForumWriteKind {
    Topic,
    Reply,
}

impl ForumWriteKind {
    const fn index(self) -> usize {
        match self {
            Self::Topic => 0,
            Self::Reply => 1,
        }
    }
}

pub(super) type ForumWriteRejection = WriteRejection;

pub struct ForumWriteLimiter {
    inner: ActorWriteLimiter<2>,
}

impl ForumWriteLimiter {
    pub fn new() -> Self {
        Self {
            inner: ActorWriteLimiter::new(
                MAX_FORUM_WRITE_ACTORS,
                [
                    WriteBudget {
                        attempts: TOPIC_ATTEMPTS,
                        window: WRITE_WINDOW,
                    },
                    WriteBudget {
                        attempts: REPLY_ATTEMPTS,
                        window: WRITE_WINDOW,
                    },
                ],
            ),
        }
    }

    pub(super) async fn check(
        &self,
        user_id: Uuid,
        kind: ForumWriteKind,
    ) -> Result<(), ForumWriteRejection> {
        self.inner.check(user_id, kind.index()).await
    }
}

impl Default for ForumWriteLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn topic_and_reply_budgets_are_independent() {
        let limiter = ForumWriteLimiter::new();
        let user = Uuid::new_v4();
        for _ in 0..TOPIC_ATTEMPTS {
            assert!(limiter.check(user, ForumWriteKind::Topic).await.is_ok());
        }
        assert!(limiter.check(user, ForumWriteKind::Topic).await.is_err());
        assert!(limiter.check(user, ForumWriteKind::Reply).await.is_ok());
    }
}
