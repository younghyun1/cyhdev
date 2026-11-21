//! The service state machine and the status it reports.

use std::sync::Arc;

use kcna_archive_core::service::{archive::Archive, models::StatusResponse};

/// Public view of where the snapshot is in its life cycle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServiceState {
    /// No snapshot exists on this machine; normal where the corpus is absent.
    NotInstalled,
    /// The first load is running.
    Loading,
    /// A snapshot is in memory and answering.
    Ready,
    /// The first load failed; a reload may repair it.
    Failed,
}

/// What the service holds. A reload of a ready service keeps `Ready` with the
/// old snapshot until the new one replaces it, so readers never see a gap.
#[derive(Clone)]
pub(super) enum Slot {
    NotInstalled,
    Loading,
    Ready(Arc<Archive>),
    Failed,
}

impl Slot {
    pub(super) fn state(&self) -> ServiceState {
        match self {
            Self::NotInstalled => ServiceState::NotInstalled,
            Self::Loading => ServiceState::Loading,
            Self::Ready(_) => ServiceState::Ready,
            Self::Failed => ServiceState::Failed,
        }
    }
}

/// State guarded by one short-lived lock; nothing slow runs while it is held.
pub(super) struct Shared {
    pub(super) slot: Slot,
    /// The newest reload failed and the previous snapshot is still serving.
    pub(super) last_reload_failed: bool,
}

/// Service state plus, when a snapshot is loaded, its corpus summary.
#[derive(Clone, Debug)]
pub struct ArchiveStatus {
    pub state: ServiceState,
    /// A reload is running while the previous snapshot keeps serving.
    pub reloading: bool,
    pub last_reload_failed: bool,
    pub snapshot: Option<StatusResponse>,
}
