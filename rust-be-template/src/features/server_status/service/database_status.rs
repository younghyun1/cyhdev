//! Briefly cached database status for the public health endpoint.
//!
//! The endpoint is unauthenticated and the frontend polls it on navigation, so each
//! hit must not cost a pool checkout and a query. One caller refreshes an expired
//! entry while concurrent callers wait on the same lock; failures are cached for the
//! same interval so an outage does not turn into a queue of doomed queries.

use std::{future::Future, time::Duration};

use tokio::{sync::Mutex, time::Instant};

/// Freshness of the cached status; the reported latency may be this old.
pub const DATABASE_STATUS_TTL: Duration = Duration::from_secs(5);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DatabaseStatus {
    pub major_version: u32,
    pub latency: Duration,
}

struct CachedStatus {
    fetched_at: Instant,
    outcome: Result<DatabaseStatus, String>,
}

/// A single-entry cache; its size is fixed regardless of traffic.
pub struct DatabaseStatusCache {
    ttl: Duration,
    entry: Mutex<Option<CachedStatus>>,
}

impl DatabaseStatusCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            entry: Mutex::new(None),
        }
    }

    /// Returns the cached outcome while fresh, otherwise runs `refresh` once and caches it.
    pub async fn get_or_refresh<F, Fut>(&self, refresh: F) -> anyhow::Result<DatabaseStatus>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = anyhow::Result<DatabaseStatus>>,
    {
        let mut entry = self.entry.lock().await;
        if let Some(cached) = entry.as_ref()
            && cached.fetched_at.elapsed() < self.ttl
        {
            return cached
                .outcome
                .clone()
                .map_err(|error| anyhow::anyhow!(error));
        }
        let outcome = refresh().await.map_err(|error| error.to_string());
        *entry = Some(CachedStatus {
            fetched_at: Instant::now(),
            outcome: outcome.clone(),
        });
        outcome.map_err(|error| anyhow::anyhow!(error))
    }
}

/// Converts `server_version_num` (major * 10000 + minor since PostgreSQL 10) to the major.
pub fn major_version(version_num: i32) -> Option<u32> {
    u32::try_from(version_num)
        .ok()
        .filter(|number| *number >= 100_000)
        .map(|number| number / 10_000)
}

#[cfg(test)]
mod tests {
    use std::{
        sync::atomic::{AtomicUsize, Ordering},
        time::Duration,
    };

    use super::{DatabaseStatus, DatabaseStatusCache, major_version};

    const STATUS: DatabaseStatus = DatabaseStatus {
        major_version: 18,
        latency: Duration::from_millis(1),
    };

    #[test]
    fn version_numbers_reduce_to_the_major_version() {
        assert_eq!(major_version(180_001), Some(18));
        assert_eq!(major_version(100_000), Some(10));
        assert_eq!(major_version(90_624), None);
        assert_eq!(major_version(-1), None);
    }

    #[tokio::test]
    async fn fresh_entries_and_failures_are_served_without_refreshing() {
        let cache = DatabaseStatusCache::new(Duration::from_secs(60));
        let calls = AtomicUsize::new(0);
        for _ in 0..3 {
            let status = cache
                .get_or_refresh(|| async {
                    calls.fetch_add(1, Ordering::SeqCst);
                    Ok(STATUS)
                })
                .await;
            assert_eq!(status.ok(), Some(STATUS));
        }
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        let failing = DatabaseStatusCache::new(Duration::from_secs(60));
        for _ in 0..3 {
            let status = failing
                .get_or_refresh(|| async {
                    calls.fetch_add(1, Ordering::SeqCst);
                    Err(anyhow::anyhow!("pool timed out"))
                })
                .await;
            assert!(status.is_err());
        }
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn expired_entries_refresh() {
        let cache = DatabaseStatusCache::new(Duration::ZERO);
        let calls = AtomicUsize::new(0);
        for _ in 0..2 {
            let status = cache
                .get_or_refresh(|| async {
                    calls.fetch_add(1, Ordering::SeqCst);
                    Ok(STATUS)
                })
                .await;
            assert!(status.is_ok());
        }
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }
}
