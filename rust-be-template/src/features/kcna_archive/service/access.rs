//! Who may read the archive: superusers, and the accounts on a stored list
//! with a copy in memory.
//!
//! Every archive request checks the list, so the check reads the copy and
//! never PostgreSQL. The copy is brought in line with the store when the
//! service starts, when an administrator reads the list, and with each change.
//! The backend is one process, so nothing else can leave the copy stale.
//! Until the first successful read the copy is empty, which admits nobody.

use std::{collections::HashSet, sync::Arc};

use async_trait::async_trait;
use tokio::sync::{Mutex, RwLock};
use tracing::{info, warn};
use uuid::Uuid;

use super::archive_service::KcnaArchiveService;
use crate::features::kcna_archive::{
    domain::reader::{ArchiveReader, MAX_READERS},
    error::ArchiveError,
};

/// Durable storage of the list. Implementations log their own failures.
#[async_trait]
pub trait ReaderStore: Send + Sync {
    /// Every listed account that is still active, newest grant first.
    async fn list(&self) -> Result<Vec<ArchiveReader>, ArchiveError>;
    /// Lists `user_id`. `false` means there is no such active account;
    /// listing an account twice is not an error.
    async fn grant(&self, user_id: Uuid) -> Result<bool, ArchiveError>;
    /// Removes `user_id`; removing an account that is not listed is not an error.
    async fn revoke(&self, user_id: Uuid) -> Result<(), ArchiveError>;
}

/// The store, the copy in memory, and the gate that orders changes.
pub(super) struct AccessPolicy {
    store: Arc<dyn ReaderStore>,
    listed: RwLock<HashSet<Uuid>>,
    /// Held across store and apply, so two changes cannot interleave and the
    /// size limit is exact.
    changes: Mutex<()>,
}

impl AccessPolicy {
    pub(super) fn new(store: Arc<dyn ReaderStore>) -> Self {
        Self {
            store,
            listed: RwLock::new(HashSet::new()),
            changes: Mutex::new(()),
        }
    }

    /// Reads the stored list and replaces the copy in memory with it.
    async fn refresh(&self) -> Result<Vec<ArchiveReader>, ArchiveError> {
        let readers = self.store.list().await?;
        let ids = readers.iter().map(|reader| reader.user_id).collect();
        *self.listed.write().await = ids;
        Ok(readers)
    }
}

impl KcnaArchiveService {
    /// Whether `user_id` is on the list. A memory read, cheap enough for every request.
    pub async fn admits(&self, user_id: Uuid) -> bool {
        self.inner.access.listed.read().await.contains(&user_id)
    }

    /// Whether anyone is listed; while nobody is, only superusers read.
    pub async fn has_readers(&self) -> bool {
        !self.inner.access.listed.read().await.is_empty()
    }

    /// Reads the stored list and brings the copy in memory in line with it.
    pub async fn readers(&self) -> Result<Vec<ArchiveReader>, ArchiveError> {
        let policy = &self.inner.access;
        let _ordered = policy.changes.lock().await;
        policy.refresh().await
    }

    /// Adds an account to the list and returns the list as it now stands.
    pub async fn grant_reader(
        &self,
        user_id: Uuid,
        actor: Uuid,
    ) -> Result<Vec<ArchiveReader>, ArchiveError> {
        let policy = &self.inner.access;
        let _ordered = policy.changes.lock().await;
        let current = policy.refresh().await?;
        if current.iter().any(|reader| reader.user_id == user_id) {
            return Ok(current);
        }
        if current.len() >= MAX_READERS {
            return Err(ArchiveError::ReaderLimit);
        }
        if !policy.store.grant(user_id).await? {
            return Err(ArchiveError::ReaderNotFound);
        }
        // Applied before the list is read again, so a failed read cannot
        // leave a stored grant without effect.
        policy.listed.write().await.insert(user_id);
        info!(actor = %actor, reader = %user_id, "KCNA archive reader added");
        policy.refresh().await
    }

    /// Removes an account from the list and returns the list as it now stands.
    pub async fn revoke_reader(
        &self,
        user_id: Uuid,
        actor: Uuid,
    ) -> Result<Vec<ArchiveReader>, ArchiveError> {
        let policy = &self.inner.access;
        let _ordered = policy.changes.lock().await;
        policy.store.revoke(user_id).await?;
        policy.listed.write().await.remove(&user_id);
        info!(actor = %actor, reader = %user_id, "KCNA archive reader removed");
        policy.refresh().await
    }

    /// Startup read. A failure leaves the list empty, so only superusers
    /// read until an administrator opens the access page, which reads it again.
    pub(super) async fn load_readers(&self) {
        match self.readers().await {
            Ok(readers) => info!(readers = readers.len(), "KCNA archive readers loaded"),
            Err(source) => warn!(
                error = %source,
                "KCNA archive readers could not be read; only superusers may read the archive"
            ),
        }
    }
}
