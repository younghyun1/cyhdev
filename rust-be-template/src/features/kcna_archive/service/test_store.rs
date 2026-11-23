//! A reader store in memory, for tests that run without PostgreSQL.

use std::{
    collections::HashMap,
    sync::atomic::{AtomicBool, Ordering},
};

use async_trait::async_trait;
use jiff::Timestamp;
use tokio::sync::Mutex;
use uuid::Uuid;

use super::access::ReaderStore;
use crate::features::kcna_archive::{domain::reader::ArchiveReader, error::ArchiveError};

/// Knows a fixed set of active accounts and can be told to fail like a lost database.
#[derive(Default)]
pub(crate) struct MemoryReaderStore {
    /// Active accounts by identifier, with the name each is shown under.
    accounts: HashMap<Uuid, String>,
    granted: Mutex<Vec<(Uuid, Timestamp)>>,
    failing: AtomicBool,
}

impl MemoryReaderStore {
    /// A store whose active accounts are `accounts`; nobody is listed yet.
    pub(crate) fn with_accounts(accounts: impl IntoIterator<Item = (Uuid, &'static str)>) -> Self {
        Self {
            accounts: accounts
                .into_iter()
                .map(|(user_id, name)| (user_id, name.to_owned()))
                .collect(),
            granted: Mutex::new(Vec::new()),
            failing: AtomicBool::new(false),
        }
    }

    pub(crate) fn fail(&self, failing: bool) {
        self.failing.store(failing, Ordering::Release);
    }

    fn check(&self) -> Result<(), ArchiveError> {
        if self.failing.load(Ordering::Acquire) {
            Err(ArchiveError::Store)
        } else {
            Ok(())
        }
    }
}

#[async_trait]
impl ReaderStore for MemoryReaderStore {
    async fn list(&self) -> Result<Vec<ArchiveReader>, ArchiveError> {
        self.check()?;
        let granted = self.granted.lock().await;
        // Newest grant first, as the database orders them.
        Ok(granted
            .iter()
            .rev()
            .filter_map(|(user_id, granted_at)| {
                self.accounts.get(user_id).map(|name| ArchiveReader {
                    user_id: *user_id,
                    user_name: name.clone(),
                    user_email: format!("{name}@example.test"),
                    granted_at: *granted_at,
                })
            })
            .collect())
    }

    async fn grant(&self, user_id: Uuid) -> Result<bool, ArchiveError> {
        self.check()?;
        if !self.accounts.contains_key(&user_id) {
            return Ok(false);
        }
        let mut granted = self.granted.lock().await;
        if !granted.iter().any(|(listed, _)| *listed == user_id) {
            granted.push((user_id, Timestamp::now()));
        }
        Ok(true)
    }

    async fn revoke(&self, user_id: Uuid) -> Result<(), ArchiveError> {
        self.check()?;
        self.granted
            .lock()
            .await
            .retain(|(listed, _)| *listed != user_id);
        Ok(())
    }
}
