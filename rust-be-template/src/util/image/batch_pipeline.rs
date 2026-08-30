//! Bounded supervisor and staging paths for photograph batch uploads.

use std::{path::PathBuf, sync::Arc};

use chrono::Utc;
use tokio::{io::AsyncWriteExt, sync::Semaphore, task::JoinSet};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    domain::photography::{batch::session::BatchSession, photographs::PhotographContext},
    init::state::ServerState,
    util::{
        image::batch_pipeline_item::process_batch_item, media::object_store::S3MediaObjectStore,
    },
};

const MAX_CONCURRENT_BATCH_ITEMS: usize = 4;

pub fn batch_root_dir() -> PathBuf {
    std::env::temp_dir().join("cyhdev-batch")
}

pub fn batch_temp_dir(batch_id: Uuid) -> PathBuf {
    batch_root_dir().join(batch_id.to_string())
}

pub fn batch_item_path(batch_id: Uuid, item_id: Uuid) -> PathBuf {
    batch_temp_dir(batch_id).join(format!("{item_id}.orig"))
}

pub async fn open_staging_file(batch_id: Uuid, item_id: Uuid) -> std::io::Result<tokio::fs::File> {
    let dir = batch_temp_dir(batch_id);
    tokio::fs::create_dir_all(&dir).await?;
    tokio::fs::File::create(batch_item_path(batch_id, item_id)).await
}

pub struct BatchPipelineItem {
    pub item_id: Uuid,
    pub file_name: Option<String>,
    pub content_type: Option<String>,
    pub comments: String,
    pub lat: f64,
    pub lon: f64,
}

/// Spawns a bounded background supervisor for one registered batch.
pub fn spawn_batch(
    state: Arc<ServerState>,
    batch: Arc<BatchSession>,
    items: Vec<BatchPipelineItem>,
    user_id: Uuid,
    context: PhotographContext,
) {
    tokio::spawn(async move {
        let batch_id = batch.batch_id;
        let store = S3MediaObjectStore::from_config(&state.aws_profile_picture_config);
        let region = state
            .aws_profile_picture_config
            .region()
            .map(|region| region.to_string())
            .unwrap_or_else(|| "us-west-1".to_string());
        let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_BATCH_ITEMS));
        let mut join_set = JoinSet::new();

        for item in items {
            let permit = match Arc::clone(&semaphore).acquire_owned().await {
                Ok(permit) => permit,
                Err(error) => {
                    error!(batch_id = %batch_id, item_id = %item.item_id, error = %error, "Batch limiter closed");
                    batch
                        .fail_item(
                            item.item_id,
                            "internal scheduling error".to_string(),
                            Utc::now(),
                        )
                        .await;
                    continue;
                }
            };
            let state = Arc::clone(&state);
            let batch = Arc::clone(&batch);
            let store = store.clone();
            let region = region.clone();
            join_set.spawn(async move {
                process_batch_item(
                    state, store, region, batch, batch_id, item, user_id, context,
                )
                .await;
                drop(permit);
            });
        }

        while let Some(result) = join_set.join_next().await {
            if let Err(error) = result {
                error!(batch_id = %batch_id, error = %error, "Batch item task failed");
            }
        }
        reconcile_items(&batch).await;
        let directory = batch_temp_dir(batch_id);
        if let Err(error) = tokio::fs::remove_dir_all(&directory).await
            && error.kind() != std::io::ErrorKind::NotFound
        {
            warn!(batch_id = %batch_id, error = %error, path = %directory.display(), "Failed to remove batch temp directory");
        }
        info!(
            batch_id = %batch_id,
            completed = batch.completed_count(),
            failed = batch.failed_count(),
            "Batch processing finished"
        );
    });
}

async fn reconcile_items(batch: &BatchSession) {
    let now = Utc::now();
    for item in batch.snapshot_items().await {
        if !item.status.is_terminal() {
            warn!(batch_id = %batch.batch_id, item_id = %item.item_id, "Batch item did not reach a terminal state");
            batch
                .fail_item(item.item_id, "processing did not complete".to_string(), now)
                .await;
        }
    }
}

pub async fn append_chunk(file: &mut tokio::fs::File, chunk: &[u8]) -> std::io::Result<()> {
    file.write_all(chunk).await
}
