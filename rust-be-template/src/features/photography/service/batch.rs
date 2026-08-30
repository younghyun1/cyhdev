use super::photography_service::PhotographyService;
use crate::{
    features::photography::service::batch_session::BatchSession,
    util::image::batch_pipeline::batch_temp_dir,
};
use chrono::{DateTime, Utc};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use tracing::warn;
use uuid::Uuid;

const TERMINAL_BATCH_TTL_SECONDS: i64 = 30 * 60;
const STUCK_BATCH_TTL_SECONDS: i64 = 6 * 60 * 60;
pub const PHOTOGRAPH_BATCH_MAX_ENTRIES: usize = 32;

pub struct BatchRegistry {
    batches: scc::HashMap<Uuid, Arc<BatchSession>>,
    count: AtomicUsize,
}

impl BatchRegistry {
    pub fn new() -> Self {
        Self {
            batches: scc::HashMap::new(),
            count: AtomicUsize::new(0),
        }
    }
}

impl Default for BatchRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PhotographyService {
    pub async fn register_batch(&self, batch: Arc<BatchSession>) -> bool {
        if self
            .batches
            .count
            .try_update(Ordering::SeqCst, Ordering::SeqCst, |current| {
                (current < PHOTOGRAPH_BATCH_MAX_ENTRIES).then_some(current + 1)
            })
            .is_err()
        {
            return false;
        }
        match self
            .batches
            .batches
            .insert_async(batch.batch_id, batch)
            .await
        {
            Ok(_) => true,
            Err(_) => {
                self.batches.count.fetch_sub(1, Ordering::SeqCst);
                false
            }
        }
    }

    pub async fn owned_batch(&self, batch_id: Uuid, requester: Uuid) -> Option<Arc<BatchSession>> {
        self.batches
            .batches
            .read_async(&batch_id, |_, batch| {
                (batch.owner == requester).then(|| Arc::clone(batch))
            })
            .await
            .flatten()
    }

    pub async fn owned_batches(&self, requester: Uuid) -> Vec<Arc<BatchSession>> {
        let mut batches = Vec::new();
        self.batches
            .batches
            .iter_async(|_, batch| {
                if batch.owner == requester {
                    batches.push(Arc::clone(batch));
                }
                true
            })
            .await;
        batches
    }

    pub async fn prune_batches(&self, now: DateTime<Utc>) -> usize {
        let now_unix = now.timestamp();
        let mut evicted = Vec::new();
        self.batches
            .batches
            .retain_async(|batch_id, batch| {
                let idle = now_unix - batch.last_activity_unix();
                let remove = (batch.is_done() && idle > TERMINAL_BATCH_TTL_SECONDS)
                    || idle > STUCK_BATCH_TTL_SECONDS;
                if remove {
                    evicted.push(*batch_id);
                }
                !remove
            })
            .await;
        self.batches
            .count
            .fetch_sub(evicted.len(), Ordering::SeqCst);
        for batch_id in &evicted {
            let batch_id = *batch_id;
            let directory = batch_temp_dir(batch_id);
            tokio::spawn(async move {
                if let Err(error) = tokio::fs::remove_dir_all(&directory).await
                    && error.kind() != std::io::ErrorKind::NotFound
                {
                    warn!(%batch_id, %error, path = %directory.display(), "Failed to remove evicted photograph batch directory");
                }
            });
        }
        evicted.len()
    }
}
