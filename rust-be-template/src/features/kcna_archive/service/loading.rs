//! Snapshot loading: the first load at startup and atomic reloads.

use std::{sync::Arc, time::Instant};

use kcna_archive_core::{service::archive::Archive, snapshot::layout::manifest_path};
use tracing::{error, info, warn};

use super::{
    archive_service::KcnaArchiveService,
    state::{Shared, Slot},
};
use crate::features::kcna_archive::error::{ArchiveError, UnavailableReason};

/// Result of one attempt to read the snapshot directory.
enum LoadOutcome {
    Loaded(Arc<Archive>),
    NotInstalled,
    /// The failure was logged where it happened.
    Failed,
}

impl KcnaArchiveService {
    /// Reads the snapshot from disk again and swaps it in atomically.
    ///
    /// The previous snapshot keeps serving until the new one is ready, and a
    /// failed reload keeps it. Only one load runs at a time; a second request
    /// gets [`ArchiveError::Busy`]. The work runs on a detached task so a
    /// client that disconnects cannot release the gate while the blocking load
    /// is still running.
    pub async fn reload(&self) -> Result<(), ArchiveError> {
        let guard = match Arc::clone(&self.inner.load_gate).try_lock_owned() {
            Ok(guard) => guard,
            Err(_) => return Err(ArchiveError::Busy),
        };
        let service = self.clone();
        let task = tokio::spawn(async move {
            let outcome = service.reload_held().await;
            drop(guard);
            outcome
        });
        match task.await {
            Ok(outcome) => outcome,
            Err(source) => {
                error!(error = %source, "KCNA archive reload task did not finish");
                Err(ArchiveError::Internal)
            }
        }
    }

    /// Applies the outcome of the startup load.
    pub(super) async fn finish_first_load(&self) {
        let slot = match self.load_snapshot().await {
            LoadOutcome::Loaded(archive) => Slot::Ready(archive),
            LoadOutcome::NotInstalled => {
                // The normal state on machines without the corpus: one line.
                info!(
                    directory = %self.inner.directory.display(),
                    "KCNA archive snapshot is not installed; archive routes report it unavailable"
                );
                Slot::NotInstalled
            }
            LoadOutcome::Failed => Slot::Failed,
        };
        self.inner.shared.write().await.slot = slot;
    }

    /// Reload body; the caller holds the load gate.
    async fn reload_held(&self) -> Result<(), ArchiveError> {
        let had_snapshot = {
            let mut shared = self.inner.shared.write().await;
            let ready = matches!(shared.slot, Slot::Ready(_));
            if !ready {
                shared.slot = Slot::Loading;
            }
            ready
        };
        let outcome = self.load_snapshot().await;
        let mut shared = self.inner.shared.write().await;
        match outcome {
            LoadOutcome::Loaded(archive) => {
                *shared = Shared {
                    slot: Slot::Ready(archive),
                    last_reload_failed: false,
                };
                Ok(())
            }
            LoadOutcome::NotInstalled if had_snapshot => {
                warn!(
                    directory = %self.inner.directory.display(),
                    "KCNA archive snapshot disappeared from disk; the loaded one keeps serving"
                );
                shared.last_reload_failed = true;
                Err(ArchiveError::ReloadFailed)
            }
            LoadOutcome::NotInstalled => {
                shared.slot = Slot::NotInstalled;
                Err(ArchiveError::Unavailable(UnavailableReason::NotInstalled))
            }
            LoadOutcome::Failed if had_snapshot => {
                shared.last_reload_failed = true;
                Err(ArchiveError::ReloadFailed)
            }
            LoadOutcome::Failed => {
                shared.slot = Slot::Failed;
                Err(ArchiveError::ReloadFailed)
            }
        }
    }

    /// Checks for the snapshot and reads it on the blocking pool. Failures are
    /// logged here, once, with structured fields.
    async fn load_snapshot(&self) -> LoadOutcome {
        let directory = self.inner.directory.clone();
        let manifest = manifest_path(&directory);
        match tokio::fs::try_exists(&manifest).await {
            Ok(true) => {}
            Ok(false) => return LoadOutcome::NotInstalled,
            Err(source) => {
                error!(
                    path = %manifest.display(),
                    error = %source,
                    "KCNA archive snapshot manifest could not be checked"
                );
                return LoadOutcome::Failed;
            }
        }
        let loader = Arc::clone(&self.inner.loader);
        let started = Instant::now();
        let path = directory.clone();
        match tokio::task::spawn_blocking(move || loader(&path)).await {
            Ok(Ok(archive)) => {
                info!(
                    directory = %directory.display(),
                    memory_bytes = archive.memory_bytes(),
                    elapsed = ?started.elapsed(),
                    "KCNA archive snapshot loaded into memory"
                );
                LoadOutcome::Loaded(Arc::new(archive))
            }
            Ok(Err(source)) => {
                error!(
                    directory = %directory.display(),
                    error = %source,
                    "KCNA archive snapshot failed to load"
                );
                LoadOutcome::Failed
            }
            Err(source) => {
                error!(
                    directory = %directory.display(),
                    error = %source,
                    "KCNA archive load task did not finish"
                );
                LoadOutcome::Failed
            }
        }
    }
}
