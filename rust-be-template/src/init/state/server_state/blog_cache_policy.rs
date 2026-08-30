use std::sync::atomic::{AtomicU64, Ordering};

pub(crate) const BLOG_POST_CACHE_MAX_ENTRIES: usize = 10_000;

#[derive(Default)]
pub(crate) struct BlogCacheMetrics {
    hits: AtomicU64,
    misses: AtomicU64,
    evictions: AtomicU64,
    rejected_admissions: AtomicU64,
    database_read_throughs: AtomicU64,
}

#[derive(Clone, Copy)]
pub(super) struct BlogCacheMetricSnapshot {
    pub(super) hits: u64,
    pub(super) misses: u64,
    pub(super) evictions: u64,
    pub(super) rejected_admissions: u64,
    pub(super) database_read_throughs: u64,
}

impl BlogCacheMetrics {
    pub(super) fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn record_eviction(&self) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn record_rejected_admission(&self) {
        self.rejected_admissions.fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn record_database_read_through(&self) {
        self.database_read_throughs
            .fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn snapshot(&self) -> BlogCacheMetricSnapshot {
        BlogCacheMetricSnapshot {
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            evictions: self.evictions.load(Ordering::Relaxed),
            rejected_admissions: self.rejected_admissions.load(Ordering::Relaxed),
            database_read_throughs: self.database_read_throughs.load(Ordering::Relaxed),
        }
    }
}
