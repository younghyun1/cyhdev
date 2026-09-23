//! Lock-free, process-wide log rate limiting.

use std::sync::atomic::{AtomicU64, Ordering};

/// Admits at most one log line per interval and counts the rest, so noisy
/// client-triggered conditions stay visible without controlling log volume.
pub(super) struct LogThrottle {
    next_emit_millis: AtomicU64,
    suppressed: AtomicU64,
}

impl LogThrottle {
    pub(super) const fn new() -> Self {
        Self {
            next_emit_millis: AtomicU64::new(0),
            suppressed: AtomicU64::new(0),
        }
    }

    /// Returns the number of suppressed events since the last admitted one
    /// when the caller may log now, otherwise counts this event and returns
    /// `None`. Exactly one concurrent caller wins each interval.
    pub(super) fn admit(&self, now_millis: u64, interval_millis: u64) -> Option<u64> {
        let next = self.next_emit_millis.load(Ordering::Acquire);
        if now_millis >= next
            && self
                .next_emit_millis
                .compare_exchange(
                    next,
                    now_millis.saturating_add(interval_millis),
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_ok()
        {
            return Some(self.suppressed.swap(0, Ordering::AcqRel));
        }
        self.suppressed.fetch_add(1, Ordering::AcqRel);
        None
    }
}

#[cfg(test)]
mod tests {
    use super::LogThrottle;

    #[test]
    fn admits_once_per_interval_and_reports_suppressed_events() {
        let throttle = LogThrottle::new();
        assert_eq!(throttle.admit(1_000, 10_000), Some(0));
        assert_eq!(throttle.admit(1_001, 10_000), None);
        assert_eq!(throttle.admit(5_000, 10_000), None);
        assert_eq!(throttle.admit(11_000, 10_000), Some(2));
        assert_eq!(throttle.admit(11_001, 10_000), None);
    }
}
