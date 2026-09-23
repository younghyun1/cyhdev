//! Bounded, lossless write-behind buffering for approximate view counters.
//!
//! Detail reads record a view by incrementing an in-memory delta instead of
//! issuing one `UPDATE` per read. A periodic flush applies every delta in one
//! batched statement per chunk. The buffer admits at most `max_entries`
//! distinct rows; existing rows keep coalescing when it is full, and a novel
//! row triggers a synchronous flush followed, if still full, by a direct
//! single-row increment, so no view is dropped and memory never grows.
//!
//! Readers that combine a database count with the pending delta use the flush
//! epoch: it is odd while a flush is between its database write and its buffer
//! settlement, so an unchanged even epoch proves the two values do not overlap.

use std::{
    collections::HashMap,
    sync::atomic::{AtomicU64, Ordering},
};

use tokio::sync::{Mutex, MutexGuard, RwLock};
use tracing::warn;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewAdmission {
    Recorded,
    Full,
    CounterSaturated,
}

/// Where a recorded view ended up.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewRecordOutcome {
    Buffered,
    Persisted,
}

/// Recording failure: an `i64` delta overflow or a failed database write.
#[derive(Debug)]
pub enum ViewRecordError<E> {
    CounterSaturated,
    Sink(E),
}

pub struct ViewDeltaBuffer {
    label: &'static str,
    max_entries: usize,
    buffer: RwLock<HashMap<Uuid, i64>>,
    saturation_events: AtomicU64,
    flush_epoch: AtomicU64,
    flush_gate: Mutex<()>,
}

impl ViewDeltaBuffer {
    pub fn new(label: &'static str, max_entries: usize) -> Self {
        Self {
            label,
            max_entries,
            buffer: RwLock::new(HashMap::new()),
            saturation_events: AtomicU64::new(0),
            flush_epoch: AtomicU64::new(0),
            flush_gate: Mutex::new(()),
        }
    }

    pub async fn try_record(&self, row_id: Uuid) -> ViewAdmission {
        let mut buffer = self.buffer.write().await;
        let has_capacity = buffer.len() < self.max_entries;
        match buffer.get_mut(&row_id) {
            Some(delta) => match delta.checked_add(1) {
                Some(next) => {
                    *delta = next;
                    ViewAdmission::Recorded
                }
                None => ViewAdmission::CounterSaturated,
            },
            None if has_capacity => {
                buffer.insert(row_id, 1);
                ViewAdmission::Recorded
            }
            None => ViewAdmission::Full,
        }
    }

    /// Views recorded for `row_id` that the database does not contain yet.
    pub async fn pending(&self, row_id: Uuid) -> i64 {
        self.buffer.read().await.get(&row_id).copied().unwrap_or(0)
    }

    /// Current flush epoch; odd while a flush is settling.
    pub fn epoch(&self) -> u64 {
        self.flush_epoch.load(Ordering::Acquire)
    }

    /// Waits until no flush is settling and returns that even epoch.
    pub async fn stable_epoch(&self) -> u64 {
        loop {
            let epoch = self.epoch();
            if epoch.is_multiple_of(2) {
                return epoch;
            }
            drop(self.flush_gate.lock().await);
        }
    }

    /// Serializes readers that need an exact database-plus-pending snapshot.
    pub async fn lock_flush(&self) -> MutexGuard<'_, ()> {
        self.flush_gate.lock().await
    }

    /// Records one view without losing it when the buffer is full.
    ///
    /// `apply` persists a batch and returns the deltas it applied; `increment`
    /// persists a single view directly when even a flush cannot free a slot.
    pub async fn record<E, Apply, Applied, Increment, Incremented>(
        &self,
        row_id: Uuid,
        apply: Apply,
        increment: Increment,
    ) -> Result<ViewRecordOutcome, ViewRecordError<E>>
    where
        Apply: FnOnce(Vec<(Uuid, i64)>) -> Applied,
        Applied: Future<Output = Result<Vec<(Uuid, i64)>, E>>,
        Increment: FnOnce(Uuid) -> Incremented,
        Incremented: Future<Output = Result<(), E>>,
    {
        match self.try_record(row_id).await {
            ViewAdmission::Recorded => return Ok(ViewRecordOutcome::Buffered),
            ViewAdmission::CounterSaturated => return Err(ViewRecordError::CounterSaturated),
            ViewAdmission::Full => self.record_saturation(),
        }
        let gate = self.flush_gate.lock().await;
        match self.try_record(row_id).await {
            ViewAdmission::Recorded => return Ok(ViewRecordOutcome::Buffered),
            ViewAdmission::CounterSaturated => return Err(ViewRecordError::CounterSaturated),
            ViewAdmission::Full => {}
        }
        self.flush_locked(&gate, apply)
            .await
            .map_err(ViewRecordError::Sink)?;
        match self.try_record(row_id).await {
            ViewAdmission::Recorded => Ok(ViewRecordOutcome::Buffered),
            ViewAdmission::CounterSaturated => Err(ViewRecordError::CounterSaturated),
            ViewAdmission::Full => {
                increment(row_id).await.map_err(ViewRecordError::Sink)?;
                Ok(ViewRecordOutcome::Persisted)
            }
        }
    }

    /// Applies every pending delta and returns the number of views persisted.
    pub async fn flush<E, Apply, Applied>(&self, apply: Apply) -> Result<u64, E>
    where
        Apply: FnOnce(Vec<(Uuid, i64)>) -> Applied,
        Applied: Future<Output = Result<Vec<(Uuid, i64)>, E>>,
    {
        let gate = self.flush_gate.lock().await;
        self.flush_locked(&gate, apply).await
    }

    /// Flush body; the guard argument proves the caller holds the flush gate.
    pub async fn flush_locked<E, Apply, Applied>(
        &self,
        _gate: &MutexGuard<'_, ()>,
        apply: Apply,
    ) -> Result<u64, E>
    where
        Apply: FnOnce(Vec<(Uuid, i64)>) -> Applied,
        Applied: Future<Output = Result<Vec<(Uuid, i64)>, E>>,
    {
        let _epoch = FlushEpochGuard::begin(&self.flush_epoch);
        // Snapshot without draining. Entries keep their admission slots while
        // persistence runs, so failed chunks remain queued without growth.
        let pending = self
            .buffer
            .read()
            .await
            .iter()
            .map(|(id, delta)| (*id, *delta))
            .collect::<Vec<_>>();
        if pending.is_empty() {
            return Ok(0);
        }
        let applied = apply(pending).await?;
        let mut flushed = 0_u64;
        let mut buffer = self.buffer.write().await;
        for (row_id, delta) in applied {
            match buffer.get_mut(&row_id) {
                Some(current) if *current > delta => *current -= delta,
                Some(_) => {
                    buffer.remove(&row_id);
                }
                None => {}
            }
            if let Ok(delta) = u64::try_from(delta) {
                flushed = flushed.saturating_add(delta);
            }
        }
        Ok(flushed)
    }

    fn record_saturation(&self) {
        let count = self
            .saturation_events
            .fetch_add(1, Ordering::Relaxed)
            .saturating_add(1);
        if count.is_power_of_two() {
            warn!(
                counter = self.label,
                saturation_events = count,
                max_entries = self.max_entries,
                "View buffer is full; using synchronous persistence"
            );
        }
    }
}

/// Marks the epoch odd for the lifetime of one flush, including early returns.
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

#[cfg(test)]
#[path = "view_delta_buffer_tests.rs"]
mod tests;
