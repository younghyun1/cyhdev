use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use tracing::warn;

pub(super) const VISITOR_BOARD_MAX_ENTRIES: usize = 10_000;
pub(super) const VISITOR_LOG_BUFFER_MAX_ENTRIES: usize = 4_096;
// Six bind values per row keeps one flush below PostgreSQL's 65,535-parameter limit.
pub(super) const VISITOR_LOG_BUFFER_MAX_EVENTS: usize = 10_000;

pub(super) fn try_reserve(counter: &AtomicUsize, max: usize) -> bool {
    counter
        .try_update(Ordering::SeqCst, Ordering::SeqCst, |current| {
            (current < max).then_some(current + 1)
        })
        .is_ok()
}

pub(super) fn reserve_up_to(counter: &AtomicUsize, requested: usize, max: usize) -> usize {
    let mut admitted = 0;
    let _ = counter.try_update(Ordering::SeqCst, Ordering::SeqCst, |current| {
        admitted = requested.min(max.saturating_sub(current));
        (admitted > 0).then_some(current + admitted)
    });
    admitted
}

pub(super) fn record_rejection(counter: &AtomicU64, cache: &'static str) {
    let total = counter.fetch_add(1, Ordering::Relaxed).saturating_add(1);
    if total.is_power_of_two() {
        warn!(cache, rejected_total = total, "Rejected cache admission at capacity");
    }
}
