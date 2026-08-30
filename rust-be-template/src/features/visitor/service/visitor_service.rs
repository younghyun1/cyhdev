//! Process-owned bounded visitor-board and write-buffer state.

use std::sync::{
    Arc,
    atomic::{AtomicU64, AtomicUsize, Ordering},
};

use crate::features::{
    geo::service::geo_service::GeoService,
    visitor::{
        domain::visit::{VisitorLogBatch, VisitorLogKey},
        repository::visitor_repository::VisitorRepository,
    },
};

pub const VISITOR_BOARD_MAX_ENTRIES: usize = 10_000;
pub const VISITOR_LOG_BUFFER_MAX_ENTRIES: usize = 4_096;
pub const VISITOR_LOG_BUFFER_MAX_EVENTS: usize = 10_000;

pub struct VisitorService {
    pub(super) repository: Arc<VisitorRepository>,
    pub(super) geo: Arc<GeoService>,
    pub(super) board: scc::HashMap<([u8; 8], [u8; 8]), u64>,
    pub(super) board_entries: AtomicUsize,
    pub(super) board_rejections: AtomicU64,
    pub(super) buffer: scc::HashMap<VisitorLogKey, VisitorLogBatch>,
    pub(super) buffer_entries: AtomicUsize,
    pub(super) pending_events: AtomicUsize,
    pub(super) buffer_rejections: AtomicU64,
    pub(super) flush_gate: tokio::sync::Mutex<()>,
}

#[derive(Clone, Copy)]
pub struct VisitorMetrics {
    pub board_entries: usize,
    pub board_rejections: u64,
    pub buffer_entries: usize,
    pub pending_events: usize,
    pub buffer_rejections: u64,
}

impl VisitorService {
    pub fn new(repository: Arc<VisitorRepository>, geo: Arc<GeoService>) -> Self {
        Self {
            repository,
            geo,
            board: scc::HashMap::new(),
            board_entries: AtomicUsize::new(0),
            board_rejections: AtomicU64::new(0),
            buffer: scc::HashMap::new(),
            buffer_entries: AtomicUsize::new(0),
            pending_events: AtomicUsize::new(0),
            buffer_rejections: AtomicU64::new(0),
            flush_gate: tokio::sync::Mutex::new(()),
        }
    }

    pub fn metrics(&self) -> VisitorMetrics {
        VisitorMetrics {
            board_entries: self.board_entries.load(Ordering::Relaxed),
            board_rejections: self.board_rejections.load(Ordering::Relaxed),
            buffer_entries: self.buffer_entries.load(Ordering::Relaxed),
            pending_events: self.pending_events.load(Ordering::Relaxed),
            buffer_rejections: self.buffer_rejections.load(Ordering::Relaxed),
        }
    }
}

pub(super) fn try_reserve(counter: &AtomicUsize, max: usize) -> bool {
    counter
        .try_update(Ordering::SeqCst, Ordering::SeqCst, |current| {
            (current < max).then_some(current + 1)
        })
        .is_ok()
}

pub(super) fn rejection(counter: &AtomicU64, cache: &'static str) {
    let total = counter.fetch_add(1, Ordering::Relaxed).saturating_add(1);
    if total.is_power_of_two() {
        tracing::warn!(
            cache,
            rejected_total = total,
            "Rejected visitor cache admission"
        );
    }
}
