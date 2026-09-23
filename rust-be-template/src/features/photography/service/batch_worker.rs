//! Bounded background batch image and persistence pipeline.

use chrono::Utc;
use std::sync::Arc;
use tokio::{sync::Semaphore, task::JoinSet};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    features::{
        accounts::service::account_service::AccountService,
        photography::{
            domain::{batch::BatchPipelineItem, photograph::PhotographContext},
            repository::photography_repository::PhotographyRepository,
            service::batch_session::BatchSession,
        },
    },
    util::{image::batch_pipeline::BatchStagingDir, media::object_store::MediaObjectStore},
};

use super::photography_service::PhotographyService;

const MAX_CONCURRENT_BATCH_ITEMS: usize = 4;

impl PhotographyService {
    pub fn spawn_batch(
        &self,
        batch: Arc<BatchSession>,
        staging: BatchStagingDir,
        items: Vec<BatchPipelineItem>,
        user_id: Uuid,
        context: PhotographContext,
    ) {
        let worker = BatchWorker {
            repository: self.repository.clone(),
            object_store: Arc::clone(&self.media.object_store),
            object_store_region: Arc::clone(&self.media.object_store_region),
            accounts: Arc::clone(&self.media.accounts),
        };
        tokio::spawn(async move {
            worker.run(batch, staging, items, user_id, context).await;
        });
    }
}

#[derive(Clone)]
pub(super) struct BatchWorker {
    pub(super) repository: PhotographyRepository,
    pub(super) object_store: Arc<dyn MediaObjectStore>,
    pub(super) object_store_region: Arc<str>,
    pub(super) accounts: Arc<AccountService>,
}

impl BatchWorker {
    async fn run(
        self,
        batch: Arc<BatchSession>,
        staging: BatchStagingDir,
        items: Vec<BatchPipelineItem>,
        user_id: Uuid,
        context: PhotographContext,
    ) {
        let batch_id = batch.batch_id;
        let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_BATCH_ITEMS));
        let mut tasks = JoinSet::new();
        for item in items {
            let permit = match Arc::clone(&semaphore).acquire_owned().await {
                Ok(permit) => permit,
                Err(error) => {
                    error!(%batch_id, item_id = %item.item_id, %error, "Batch limiter closed");
                    batch
                        .fail_item(
                            item.item_id,
                            "internal scheduling error".to_owned(),
                            Utc::now(),
                        )
                        .await;
                    continue;
                }
            };
            let worker = self.clone();
            let batch = Arc::clone(&batch);
            let directory = staging.path().to_path_buf();
            tasks.spawn(async move {
                worker
                    .process(Arc::clone(&batch), &directory, item, user_id, context)
                    .await;
                drop(permit);
            });
        }
        while let Some(result) = tasks.join_next().await {
            if let Err(error) = result {
                error!(%batch_id, %error, "Batch item task failed");
            }
        }
        for item in batch.snapshot_items().await {
            if !item.status.is_terminal() {
                batch
                    .fail_item(
                        item.item_id,
                        "processing did not complete".to_owned(),
                        Utc::now(),
                    )
                    .await;
            }
        }
        if let Err(error) = staging.remove().await
            && error.kind() != std::io::ErrorKind::NotFound
        {
            warn!(%batch_id, %error, "Failed to remove batch staging directory");
        }
        info!(%batch_id, completed = batch.completed_count(), failed = batch.failed_count(), "Batch processing finished");
    }
}
