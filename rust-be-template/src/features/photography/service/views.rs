//! Bounded, lossless photograph view buffering.

use std::{
    collections::HashMap,
    sync::atomic::{AtomicU64, Ordering},
};

use tokio::sync::{Mutex, RwLock};
use tracing::warn;
use uuid::Uuid;

use super::super::{domain::photograph::PhotographDetail, error::PhotographyError};
use super::photography_service::PhotographyService;

pub const PHOTOGRAPH_VIEW_BUFFER_MAX_ENTRIES: usize = 8_192;
const VIEW_DETAIL_OPTIMISTIC_RETRIES: usize = 3;

enum RecordOutcome {
    Buffered,
    Persisted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Admission {
    Recorded,
    Full,
    CounterSaturated,
}

pub struct PhotographViewBuffer {
    buffer: RwLock<HashMap<Uuid, i64>>,
    saturation_events: AtomicU64,
    flush_epoch: AtomicU64,
    flush_gate: Mutex<()>,
}

impl PhotographViewBuffer {
    pub fn new() -> Self {
        Self {
            buffer: RwLock::new(HashMap::new()),
            saturation_events: AtomicU64::new(0),
            flush_epoch: AtomicU64::new(0),
            flush_gate: Mutex::new(()),
        }
    }

    async fn try_record(&self, photograph_id: Uuid) -> Admission {
        let mut buffer = self.buffer.write().await;
        let has_capacity = buffer.len() < PHOTOGRAPH_VIEW_BUFFER_MAX_ENTRIES;
        match buffer.get_mut(&photograph_id) {
            Some(delta) => match delta.checked_add(1) {
                Some(next) => {
                    *delta = next;
                    Admission::Recorded
                }
                None => Admission::CounterSaturated,
            },
            None if has_capacity => {
                buffer.insert(photograph_id, 1);
                Admission::Recorded
            }
            None => Admission::Full,
        }
    }

    async fn pending(&self, photograph_id: Uuid) -> i64 {
        self.buffer.read().await.get(&photograph_id).copied().unwrap_or(0)
    }

    async fn stable_epoch(&self) -> u64 {
        loop {
            let epoch = self.flush_epoch.load(Ordering::Acquire);
            if epoch.is_multiple_of(2) {
                return epoch;
            }
            let gate = self.flush_gate.lock().await;
            drop(gate);
        }
    }

    fn record_saturation(&self) {
        let count = self.saturation_events.fetch_add(1, Ordering::Relaxed).saturating_add(1);
        if count.is_power_of_two() {
            warn!(saturation_events = count, max_entries = PHOTOGRAPH_VIEW_BUFFER_MAX_ENTRIES,
                "Photograph view buffer is full; using synchronous persistence");
        }
    }
}

struct FlushEpochGuard<'a>(&'a AtomicU64);

impl<'a> FlushEpochGuard<'a> {
    fn begin(epoch: &'a AtomicU64) -> Self {
        epoch.fetch_add(1, Ordering::AcqRel);
        Self(epoch)
    }
}

impl Drop for FlushEpochGuard<'_> {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::Release);
    }
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
            let mut detail = self.repository.photograph_detail(photograph_id, viewer).await?;
            let persisted = if recorded {
                false
            } else {
                recorded = true;
                matches!(self.record_view_lossless(photograph_id).await?, RecordOutcome::Persisted)
            };
            let pending = self.views.pending(photograph_id).await;
            let epoch_after = self.views.flush_epoch.load(Ordering::Acquire);
            if !persisted && epoch_before == epoch_after && epoch_after.is_multiple_of(2) {
                detail.photograph.photograph_view_count = detail.photograph.photograph_view_count
                    .checked_add(pending)
                    .ok_or(PhotographyError::ViewCounterSaturated)?;
                return Ok(detail);
            }
        }

        // A hot flush loop falls back to one serialized snapshot. Ordinary
        // buffered increments may continue because they do not change the DB.
        let _gate = self.views.flush_gate.lock().await;
        let mut detail = self.repository.photograph_detail(photograph_id, viewer).await?;
        let pending = self.views.pending(photograph_id).await;
        detail.photograph.photograph_view_count = detail.photograph.photograph_view_count
            .checked_add(pending)
            .ok_or(PhotographyError::ViewCounterSaturated)?;
        Ok(detail)
    }

    async fn record_view_lossless(&self, photograph_id: Uuid) -> Result<RecordOutcome, PhotographyError> {
        match self.views.try_record(photograph_id).await {
            Admission::Recorded => return Ok(RecordOutcome::Buffered),
            Admission::CounterSaturated => return Err(PhotographyError::ViewCounterSaturated),
            Admission::Full => self.views.record_saturation(),
        }

        let _gate = self.views.flush_gate.lock().await;
        match self.views.try_record(photograph_id).await {
            Admission::Recorded => return Ok(RecordOutcome::Buffered),
            Admission::CounterSaturated => return Err(PhotographyError::ViewCounterSaturated),
            Admission::Full => {}
        }
        self.flush_views_locked().await?;
        match self.views.try_record(photograph_id).await {
            Admission::Recorded => Ok(RecordOutcome::Buffered),
            Admission::CounterSaturated => Err(PhotographyError::ViewCounterSaturated),
            Admission::Full => {
                self.repository.increment_view(photograph_id).await?;
                Ok(RecordOutcome::Persisted)
            }
        }
    }

    pub async fn flush_views(&self) -> Result<u64, PhotographyError> {
        let _gate = self.views.flush_gate.lock().await;
        self.flush_views_locked().await
    }

    async fn flush_views_locked(&self) -> Result<u64, PhotographyError> {
        let _epoch = FlushEpochGuard::begin(&self.views.flush_epoch);
        // Snapshot without draining. Entries retain their admission slots while
        // persistence runs, so failed chunks remain queued without growth.
        let pending = self.views.buffer.read().await.iter().map(|(id, delta)| (*id, *delta)).collect::<Vec<_>>();
        let applied = self.repository.apply_view_deltas(&pending).await?;
        let mut flushed = 0_u64;
        let mut buffer = self.views.buffer.write().await;
        for (photograph_id, delta) in applied {
            match buffer.get_mut(&photograph_id) {
                Some(current) if *current > delta => *current -= delta,
                Some(_) => {
                    buffer.remove(&photograph_id);
                }
                None => {}
            }
            if let Ok(delta) = u64::try_from(delta) {
                flushed = flushed.saturating_add(delta);
            }
        }
        Ok(flushed)
    }
}

impl Default for PhotographViewBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{Admission, PHOTOGRAPH_VIEW_BUFFER_MAX_ENTRIES, PhotographViewBuffer};
    use uuid::Uuid;

    #[tokio::test]
    async fn full_buffer_still_accepts_an_existing_key() {
        let views = PhotographViewBuffer::new();
        let existing = Uuid::from_u128(1);
        for value in 1..=PHOTOGRAPH_VIEW_BUFFER_MAX_ENTRIES {
            let id = Uuid::from_u128(value as u128);
            assert_eq!(views.try_record(id).await, Admission::Recorded);
        }
        assert_eq!(views.try_record(existing).await, Admission::Recorded);
        assert_eq!(views.pending(existing).await, 2);
        assert_eq!(views.try_record(Uuid::now_v7()).await, Admission::Full);
    }
}
