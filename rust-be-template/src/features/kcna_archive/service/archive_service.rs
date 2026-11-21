//! Clonable handle owning the snapshot state of the archive.
//!
//! The service keeps one snapshot in memory behind a short-lived lock. Queries
//! clone the `Arc` out of the lock and run on the blocking pool under a small
//! semaphore, so a reload replaces the snapshot atomically while queries that
//! already started finish on the old one. Peak memory during a reload is
//! therefore about two snapshots.

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use kcna_archive_core::{
    error::Error as CoreError,
    service::archive::{Archive, ServiceError},
};
use tokio::sync::{Mutex, RwLock, Semaphore};
use tracing::error;

use super::{
    config::KcnaArchiveConfig,
    state::{ArchiveStatus, Shared, Slot},
};
use crate::features::kcna_archive::error::{ArchiveError, UnavailableReason};

/// Archive queries that may run on the blocking pool at once. Each one is
/// CPU-bound for tens of milliseconds, so four keep several administrators
/// responsive while one person's burst of searches cannot occupy the whole
/// blocking pool shared with image processing and password hashing.
pub const MAX_CONCURRENT_QUERIES: usize = 4;

/// Reads a snapshot directory into memory. Production uses
/// `Archive::load_into_memory`; tests substitute a gated loader.
pub type SnapshotLoader = Arc<dyn Fn(&Path) -> Result<Archive, CoreError> + Send + Sync>;

/// State shared by every clone of the handle.
pub(super) struct Inner {
    pub(super) directory: PathBuf,
    pub(super) loader: SnapshotLoader,
    pub(super) shared: RwLock<Shared>,
    /// Held for the whole of a load or reload, so only one runs at a time.
    pub(super) load_gate: Arc<Mutex<()>>,
    pub(super) queries: Arc<Semaphore>,
}

/// Handle to the in-memory archive; cloning shares the same state.
#[derive(Clone)]
pub struct KcnaArchiveService {
    pub(super) inner: Arc<Inner>,
}

impl KcnaArchiveService {
    /// Starts the service from `KCNA_ARCHIVE_DIR` and returns at once; the
    /// snapshot loads on the blocking pool while the server accepts requests.
    pub async fn start_from_environment() -> Self {
        let config = KcnaArchiveConfig::from_environment();
        Self::start(config.directory, Arc::new(Archive::load_into_memory)).await
    }

    /// Starts the service on `directory` with an explicit loader.
    pub(crate) async fn start(directory: PathBuf, loader: SnapshotLoader) -> Self {
        let load_gate = Arc::new(Mutex::new(()));
        // A fresh mutex is free, so this returns immediately. The first load
        // holds the gate so an early reload request gets a conflict.
        let first_load = Arc::clone(&load_gate).lock_owned().await;
        let service = Self {
            inner: Arc::new(Inner {
                directory,
                loader,
                shared: RwLock::new(Shared {
                    slot: Slot::Loading,
                    last_reload_failed: false,
                }),
                load_gate,
                queries: Arc::new(Semaphore::new(MAX_CONCURRENT_QUERIES)),
            }),
        };
        let background = service.clone();
        tokio::spawn(async move {
            background.finish_first_load().await;
            drop(first_load);
        });
        service
    }

    /// The service state and, when ready, the snapshot's corpus summary.
    pub async fn status(&self) -> Result<ArchiveStatus, ArchiveError> {
        let (slot, last_reload_failed) = {
            let shared = self.inner.shared.read().await;
            (shared.slot.clone(), shared.last_reload_failed)
        };
        let state = slot.state();
        // A held gate while a snapshot serves means a reload is replacing it.
        let reloading = matches!(slot, Slot::Ready(_)) && self.inner.load_gate.try_lock().is_err();
        let snapshot = match slot {
            Slot::Ready(archive) => Some(self.run(archive, |archive| archive.status()).await?),
            Slot::NotInstalled | Slot::Loading | Slot::Failed => None,
        };
        Ok(ArchiveStatus {
            state,
            reloading,
            last_reload_failed,
            snapshot,
        })
    }

    /// Runs one read on the loaded snapshot, or says why there is none.
    pub(super) async fn query<T, F>(&self, operation: F) -> Result<T, ArchiveError>
    where
        T: Send + 'static,
        F: FnOnce(&Archive) -> Result<T, ServiceError> + Send + 'static,
    {
        let archive = self.ready().await?;
        self.run(archive, operation).await
    }

    async fn ready(&self) -> Result<Arc<Archive>, ArchiveError> {
        let shared = self.inner.shared.read().await;
        match &shared.slot {
            Slot::Ready(archive) => Ok(Arc::clone(archive)),
            Slot::NotInstalled => Err(ArchiveError::Unavailable(UnavailableReason::NotInstalled)),
            Slot::Loading => Err(ArchiveError::Unavailable(UnavailableReason::Loading)),
            Slot::Failed => Err(ArchiveError::Unavailable(UnavailableReason::Failed)),
        }
    }

    /// Executes `operation` on the blocking pool under the query semaphore.
    /// The permit moves into the blocking task, so it is held until the work
    /// ends even when the request that asked for it is dropped.
    async fn run<T, F>(&self, archive: Arc<Archive>, operation: F) -> Result<T, ArchiveError>
    where
        T: Send + 'static,
        F: FnOnce(&Archive) -> Result<T, ServiceError> + Send + 'static,
    {
        let permit = match Arc::clone(&self.inner.queries).acquire_owned().await {
            Ok(permit) => permit,
            Err(source) => {
                error!(error = %source, "KCNA archive query semaphore is closed");
                return Err(ArchiveError::Internal);
            }
        };
        let joined = tokio::task::spawn_blocking(move || {
            let outcome = operation(&archive);
            drop(permit);
            outcome
        })
        .await;
        match joined {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(failure)) => Err(map_core_error(failure)),
            Err(source) => {
                error!(error = %source, "KCNA archive query task did not finish");
                Err(ArchiveError::Internal)
            }
        }
    }
}

/// Translates the core service's error into the feature error. An index
/// failure is logged here, where the context is known, and reported opaquely.
fn map_core_error(failure: ServiceError) -> ArchiveError {
    match failure {
        ServiceError::BadRequest => ArchiveError::BadRequest,
        ServiceError::NotFound => ArchiveError::NotFound,
        ServiceError::Unavailable(source) => {
            error!(error = %source, "KCNA archive query failed");
            ArchiveError::Internal
        }
    }
}
