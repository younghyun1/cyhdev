//! Bounded, lossless photograph view buffering.

use uuid::Uuid;

use super::super::{domain::photograph::PhotographDetail, error::PhotographyError};
use super::photography_service::PhotographyService;
use crate::util::view_delta_buffer::{ViewDeltaBuffer, ViewRecordError, ViewRecordOutcome};

pub const PHOTOGRAPH_VIEW_BUFFER_MAX_ENTRIES: usize = 8_192;
const VIEW_DETAIL_OPTIMISTIC_RETRIES: usize = 3;

pub fn photograph_view_buffer() -> ViewDeltaBuffer {
    ViewDeltaBuffer::new("photograph", PHOTOGRAPH_VIEW_BUFFER_MAX_ENTRIES)
}

impl PhotographyService {
    pub(super) async fn photograph_detail_with_view(
        &self,
        photograph_id: Uuid,
        viewer: Option<Uuid>,
    ) -> Result<PhotographDetail, PhotographyError> {
        let mut recorded = false;
        for _ in 0..VIEW_DETAIL_OPTIMISTIC_RETRIES {
            let epoch_before = self.views.stable_epoch().await;
            let mut detail = self
                .repository
                .photograph_detail(photograph_id, viewer)
                .await?;
            let persisted = if recorded {
                false
            } else {
                recorded = true;
                self.record_view_lossless(photograph_id).await? == ViewRecordOutcome::Persisted
            };
            let pending = self.views.pending(photograph_id).await;
            let epoch_after = self.views.epoch();
            if !persisted && epoch_before == epoch_after && epoch_after.is_multiple_of(2) {
                detail.photograph.photograph_view_count = detail
                    .photograph
                    .photograph_view_count
                    .checked_add(pending)
                    .ok_or(PhotographyError::ViewCounterSaturated)?;
                return Ok(detail);
            }
        }

        // A hot flush loop falls back to one serialized snapshot. Ordinary
        // buffered increments may continue because they do not change the DB.
        let _gate = self.views.lock_flush().await;
        let mut detail = self
            .repository
            .photograph_detail(photograph_id, viewer)
            .await?;
        let pending = self.views.pending(photograph_id).await;
        detail.photograph.photograph_view_count = detail
            .photograph
            .photograph_view_count
            .checked_add(pending)
            .ok_or(PhotographyError::ViewCounterSaturated)?;
        Ok(detail)
    }

    async fn record_view_lossless(
        &self,
        photograph_id: Uuid,
    ) -> Result<ViewRecordOutcome, PhotographyError> {
        self.views
            .record(
                photograph_id,
                |pending| async move { self.repository.apply_view_deltas(&pending).await },
                |photograph_id| self.repository.increment_view(photograph_id),
            )
            .await
            .map_err(|error| match error {
                ViewRecordError::CounterSaturated => PhotographyError::ViewCounterSaturated,
                ViewRecordError::Sink(error) => error,
            })
    }

    /// Persists every buffered photograph view; safe to call from shutdown.
    pub async fn flush_views(&self) -> Result<u64, PhotographyError> {
        self.views
            .flush(|pending| async move { self.repository.apply_view_deltas(&pending).await })
            .await
    }
}
