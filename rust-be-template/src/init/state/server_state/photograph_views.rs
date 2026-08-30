//! `ServerState` accessors for the in-memory photograph view-count cache.
//!
//! Views are a high-frequency, low-value write: a DB `UPDATE` per detail open is
//! wasteful. Instead each view increments an in-RAM delta under a `tokio::RwLock`
//! ([`record_view`]) and a periodic job ([`flush_photograph_views`]) folds the
//! accumulated deltas into `photographs.photograph_view_count`. The buffer is
//! drained on every flush and has a hard distinct-photograph cap.
//!
//! Loss policy: on a DB error, deltas are re-admitted within the same cap; a
//! concurrent refill can therefore drop approximate view counts. Deltas whose `UPDATE` matches zero rows are dropped, not
//! requeued: that only happens once the photograph has been deleted, and
//! requeueing a never-matching id would leak the buffer forever. A process
//! crash loses at most one flush window, acceptable for naive view counts.

use std::collections::HashMap;
use std::sync::atomic::Ordering;

use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use tracing::warn;
use uuid::Uuid;

use crate::schema::photographs;

use super::ServerState;

/// Maximum number of distinct photograph counters pending in one flush window.
const PHOTOGRAPH_VIEW_BUFFER_MAX_ENTRIES: usize = 8_192;

impl ServerState {
    /// Record a single view for `photograph_id` in the RAM buffer and return the
    /// running pending delta (including this view), so the caller can present a
    /// live count of `persisted_base + pending` without a DB round trip.
    pub async fn record_view(&self, photograph_id: Uuid) -> i64 {
        let mut buffer = self.photograph_view_buffer.write().await;
        let has_capacity = buffer.len() < PHOTOGRAPH_VIEW_BUFFER_MAX_ENTRIES;
        match buffer.get_mut(&photograph_id) {
            Some(delta) => {
                *delta = delta.saturating_add(1);
                *delta
            }
            None if has_capacity => {
                buffer.insert(photograph_id, 1);
                1
            }
            None => {
                let rejected = self
                    .photograph_view_rejected_events
                    .fetch_add(1, Ordering::Relaxed)
                    .saturating_add(1);
                if rejected.is_power_of_two() {
                    warn!(
                        rejected_events = rejected,
                        max_entries = PHOTOGRAPH_VIEW_BUFFER_MAX_ENTRIES,
                        "Dropped photograph view count because the buffer is full"
                    );
                }
                0
            }
        }
    }

    /// Drain buffered view deltas and fold them into the persisted counters.
    /// Returns the total number of views flushed.
    pub async fn flush_photograph_views(&self) -> anyhow::Result<u64> {
        let pending: HashMap<Uuid, i64> = {
            let mut buffer = self.photograph_view_buffer.write().await;
            std::mem::take(&mut *buffer)
        };
        if pending.is_empty() {
            return Ok(0);
        }

        let mut conn = match self.get_conn().await {
            Ok(conn) => conn,
            Err(e) => {
                self.requeue_photograph_views(pending).await;
                return Err(e);
            }
        };

        let mut flushed: u64 = 0;
        let mut failed: HashMap<Uuid, i64> = HashMap::new();
        for (photograph_id, delta) in pending {
            if delta <= 0 {
                continue;
            }
            let res = diesel::update(
                photographs::table.filter(photographs::photograph_id.eq(photograph_id)),
            )
            .set(photographs::photograph_view_count.eq(photographs::photograph_view_count + delta))
            .execute(&mut conn)
            .await;
            match res {
                Ok(_) => flushed = flushed.saturating_add(delta as u64),
                Err(e) => {
                    warn!(
                        photograph_id = %photograph_id,
                        error = ?e,
                        "Failed to flush photograph view delta; requeueing"
                    );
                    failed.insert(photograph_id, delta);
                }
            }
        }
        drop(conn);

        if !failed.is_empty() {
            self.requeue_photograph_views(failed).await;
        }

        Ok(flushed)
    }

    /// Merge deltas back into the buffer after a failed flush so no views are lost.
    async fn requeue_photograph_views(&self, pending: HashMap<Uuid, i64>) {
        let mut buffer = self.photograph_view_buffer.write().await;
        for (photograph_id, delta) in pending {
            let has_capacity = buffer.len() < PHOTOGRAPH_VIEW_BUFFER_MAX_ENTRIES;
            match buffer.get_mut(&photograph_id) {
                Some(existing) => *existing = existing.saturating_add(delta),
                None if has_capacity => {
                    buffer.insert(photograph_id, delta);
                }
                None => {
                    let rejected = u64::try_from(delta.max(0)).unwrap_or(u64::MAX);
                    self.photograph_view_rejected_events
                        .fetch_add(rejected, Ordering::Relaxed);
                }
            }
        }
    }
}
