//! Write-behind blog view counting.
//!
//! A detail read records its view in the bounded buffer instead of updating
//! the post row, so reads neither hold the post stripe for a write nor make
//! every read a non-HOT row update. The scheduled flush and graceful shutdown
//! call [`BlogService::flush_views`].

use tracing::warn;
use uuid::Uuid;

use super::super::error::BlogError;
use super::blog_service::BlogService;
use crate::util::view_delta_buffer::{ViewRecordError, ViewRecordOutcome};

impl BlogService {
    /// Records one view and returns the count to present.
    ///
    /// `database_count` was read after `epoch_before` was observed. When no
    /// flush ran in between, database plus pending is exact; otherwise the
    /// count is re-read under the flush gate so a settled delta is not counted
    /// twice. View counting is best effort: a failure keeps the read count.
    pub(super) async fn record_and_count_view(
        &self,
        post_id: Uuid,
        database_count: i64,
        epoch_before: u64,
    ) -> i64 {
        match self.record_view(post_id).await {
            Ok(ViewRecordOutcome::Buffered | ViewRecordOutcome::Persisted) => {}
            Err(error) => {
                warn!(%post_id, %error, "Blog detail view was not recorded");
                return database_count;
            }
        }
        let pending = self.views.pending(post_id).await;
        if self.views.epoch() == epoch_before && epoch_before.is_multiple_of(2) {
            return database_count.saturating_add(pending);
        }
        let _gate = self.views.lock_flush().await;
        match self.repository.post_view_count(post_id).await {
            Ok(Some(count)) => count.saturating_add(self.views.pending(post_id).await),
            Ok(None) => database_count,
            Err(error) => {
                warn!(%post_id, %error, "Blog view count re-read failed");
                database_count.saturating_add(pending)
            }
        }
    }

    async fn record_view(&self, post_id: Uuid) -> Result<ViewRecordOutcome, BlogError> {
        self.views
            .record(
                post_id,
                |pending| async move { self.repository.apply_view_deltas(&pending).await },
                |post_id| self.repository.increment_view(post_id),
            )
            .await
            .map_err(|error| match error {
                ViewRecordError::CounterSaturated => {
                    BlogError::Invariant("blog view delta exceeded the counter range")
                }
                ViewRecordError::Sink(error) => error,
            })
    }

    /// Persists every buffered blog view and returns how many were written.
    ///
    /// The minute scheduler calls this, and a graceful-shutdown hook should
    /// await it after the listener stops accepting requests so buffered views
    /// are not lost with the process.
    pub async fn flush_views(&self) -> Result<u64, BlogError> {
        self.views
            .flush(|pending| async move { self.repository.apply_view_deltas(&pending).await })
            .await
    }
}
