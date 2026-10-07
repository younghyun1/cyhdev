//! Fresh, bounded database probes for the public health endpoint.
//!
//! No result is retained between requests. A detached probe keeps its admission
//! permit until the database operation completes, even if its HTTP caller times
//! out or disconnects, so cancellation cannot bypass the concurrency bound.

use std::{future::Future, sync::Arc, time::Duration};

use tokio::sync::Semaphore;

/// Health probes cannot occupy more than four database operations at once.
pub const DATABASE_STATUS_MAX_PROBES: usize = 4;
/// HTTP callers stop waiting after this deadline; admitted probes finish normally.
pub const DATABASE_STATUS_DEADLINE: Duration = Duration::from_secs(2);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DatabaseStatus {
    pub major_version: u32,
    pub latency: Duration,
}

/// Fixed concurrency admission without a queue or retained measurements.
pub struct DatabaseStatusProbe {
    slots: Arc<Semaphore>,
    deadline: Duration,
}

impl DatabaseStatusProbe {
    pub fn new() -> Self {
        Self {
            slots: Arc::new(Semaphore::new(DATABASE_STATUS_MAX_PROBES)),
            deadline: DATABASE_STATUS_DEADLINE,
        }
    }

    /// Runs one fresh probe, refusing immediately when all probe slots are occupied.
    pub async fn measure<F, Fut>(&self, refresh: F) -> anyhow::Result<DatabaseStatus>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = anyhow::Result<DatabaseStatus>> + Send + 'static,
    {
        let permit = match Arc::clone(&self.slots).try_acquire_owned() {
            Ok(permit) => permit,
            Err(_) => return Err(anyhow::anyhow!("database status probes are busy")),
        };
        // Dropping a Diesel future can leave its PostgreSQL query in progress.
        // Keep the task and its permit alive instead of returning that work to the
        // pool or admitting more probes while it is still running.
        let task = tokio::spawn(async move {
            let _permit = permit;
            refresh().await
        });
        match tokio::time::timeout(self.deadline, task).await {
            Ok(Ok(outcome)) => outcome,
            Ok(Err(error)) => Err(anyhow::anyhow!("database status probe failed: {error}")),
            Err(_) => Err(anyhow::anyhow!("database status probe timed out")),
        }
    }
}

impl Default for DatabaseStatusProbe {
    fn default() -> Self {
        Self::new()
    }
}

/// Converts `server_version_num` (major * 10000 + minor since PostgreSQL 10) to the major.
pub fn major_version(version_num: i32) -> Option<u32> {
    u32::try_from(version_num)
        .ok()
        .filter(|number| *number >= 100_000)
        .map(|number| number / 10_000)
}

#[cfg(test)]
#[path = "database_status_tests.rs"]
mod tests;
