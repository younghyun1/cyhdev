//! Bounded background batch image and persistence pipeline.

use chrono::Utc;
use std::{path::PathBuf, sync::Arc};
use tokio::{sync::Semaphore, task::JoinSet};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    features::{
        accounts::service::account_service::AccountService,
        photography::{
            domain::{
                batch::{BatchPipelineItem, ProcessingStatus},
                photograph::{NewPhotograph, Photograph, PhotographContext},
            },
            error::PhotographyError,
            repository::photography_repository::PhotographyRepository,
            service::batch_session::BatchSession,
        },
    },
    util::{
        image::{
            batch_pipeline::{batch_item_path, batch_temp_dir},
            exif_utils::extract_exif_shot_at_from_path,
            image_variant::{CyhdevImageType, IMAGE_ENCODING_FORMAT},
            map_image_format_to_db_enum::map_image_format_to_str,
            process_uploaded_image_files::process_uploaded_image_files,
        },
        media::{
            object_store::{MediaObjectStore, ObjectLocation},
            persistence::{
                CleanupFailure, MediaWriteError, PendingMediaObject, PersistedMedia,
                persist_media_objects,
            },
        },
        s3::AWS_S3_BUCKET_NAME,
    },
};

use super::media::enqueue_photograph_compensation_cleanup;
use super::photography_service::PhotographyService;

const MAX_CONCURRENT_BATCH_ITEMS: usize = 4;

impl PhotographyService {
    pub fn spawn_batch(
        &self,
        batch: Arc<BatchSession>,
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
            worker.run(batch, items, user_id, context).await;
        });
    }
}

#[derive(Clone)]
struct BatchWorker {
    repository: PhotographyRepository,
    object_store: Arc<dyn MediaObjectStore>,
    object_store_region: Arc<str>,
    accounts: Arc<AccountService>,
}

impl BatchWorker {
    async fn run(
        self,
        batch: Arc<BatchSession>,
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
            tasks.spawn(async move {
                worker
                    .process(Arc::clone(&batch), batch_id, item, user_id, context)
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
        let directory = batch_temp_dir(batch_id);
        if let Err(error) = tokio::fs::remove_dir_all(&directory).await
            && error.kind() != std::io::ErrorKind::NotFound
        {
            warn!(%batch_id, %error, path = %directory.display(), "Failed to remove batch directory");
        }
        info!(%batch_id, completed = batch.completed_count(), failed = batch.failed_count(), "Batch processing finished");
    }

    async fn process(
        &self,
        batch: Arc<BatchSession>,
        batch_id: Uuid,
        item: BatchPipelineItem,
        user_id: Uuid,
        context: PhotographContext,
    ) {
        let item_id = item.item_id;
        let source_path = batch_item_path(batch_id, item_id);
        let mut guard = StagedFileGuard(Some(source_path.clone()));
        self.process_staged(batch, batch_id, item, user_id, context, source_path.clone())
            .await;
        match tokio::fs::remove_file(&source_path).await {
            Ok(()) => guard.disarm(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => guard.disarm(),
            Err(error) => {
                warn!(%batch_id, %item_id, %error, path = %source_path.display(), "Failed to remove staged batch file; synchronous drop cleanup will retry")
            }
        }
    }

    async fn process_staged(
        &self,
        batch: Arc<BatchSession>,
        batch_id: Uuid,
        item: BatchPipelineItem,
        user_id: Uuid,
        context: PhotographContext,
        source_path: PathBuf,
    ) {
        let item_id = item.item_id;
        batch
            .set_status(item_id, ProcessingStatus::Encoding, Utc::now())
            .await;
        let exif_path = source_path.clone();
        let shot_at =
            match tokio::task::spawn_blocking(move || extract_exif_shot_at_from_path(&exif_path))
                .await
            {
                Ok(Ok(value)) => value,
                Ok(Err(error)) => {
                    warn!(%batch_id, %item_id, %error, "Failed batch EXIF parse");
                    None
                }
                Err(error) => {
                    error!(%batch_id, %item_id, %error, "Batch EXIF worker failed");
                    None
                }
            };
        let mut outputs = match process_uploaded_image_files(
            &source_path,
            None,
            vec![CyhdevImageType::Photograph, CyhdevImageType::Thumbnail],
        )
        .await
        {
            Ok(outputs) => outputs.into_iter(),
            Err(error) => {
                fail(&batch, item_id, "encode", &error).await;
                return;
            }
        };
        let main = match outputs.next() {
            Some(value) => value,
            None => {
                batch
                    .fail_item(
                        item_id,
                        "encoder produced no main image".to_owned(),
                        Utc::now(),
                    )
                    .await;
                return;
            }
        };
        let thumbnail = match outputs.next() {
            Some(value) => value,
            None => {
                batch
                    .fail_item(
                        item_id,
                        "encoder produced no thumbnail".to_owned(),
                        Utc::now(),
                    )
                    .await;
                return;
            }
        };
        let (extension, image_type) = map_image_format_to_str(IMAGE_ENCODING_FORMAT);
        let main_location =
            ObjectLocation::new(AWS_S3_BUCKET_NAME, format!("images/{item_id}.{extension}"));
        let thumbnail_location = ObjectLocation::new(
            AWS_S3_BUCKET_NAME,
            format!("thumbnails/{item_id}.{extension}"),
        );
        let object_url = main_location.public_s3_url(&self.object_store_region);
        let thumbnail_url = thumbnail_location.public_s3_url(&self.object_store_region);
        let pending = [
            PendingMediaObject {
                location: main_location,
                content_type: "image/avif".to_owned(),
                source: main.path_buf(),
            },
            PendingMediaObject {
                location: thumbnail_location,
                content_type: "image/avif".to_owned(),
                source: thumbnail.path_buf(),
            },
        ];
        batch
            .set_status(item_id, ProcessingStatus::Uploading, Utc::now())
            .await;
        let repository = self.repository.clone();
        let persistence_batch = Arc::clone(&batch);
        let insert_url = object_url.clone();
        let insert_thumb = thumbnail_url.clone();
        let result = persist_media_objects::<Photograph, PhotographyError, _>(
            self.object_store.as_ref(),
            &pending,
            async move {
                persistence_batch
                    .set_status(item_id, ProcessingStatus::Persisting, Utc::now())
                    .await;
                let photograph = repository
                    .insert_photograph(
                        user_id,
                        NewPhotograph {
                            user_id,
                            photograph_shot_at: shot_at,
                            photograph_image_type: image_type,
                            photograph_context: context,
                            photograph_is_on_cloud: true,
                            photograph_link: insert_url,
                            photograph_comments: item.comments,
                            photograph_lat: item.latitude,
                            photograph_lon: item.longitude,
                            photograph_thumbnail_link: insert_thumb,
                        },
                    )
                    .await?;
                Ok(PersistedMedia::new(photograph, Vec::new()))
            },
        )
        .await;
        match result {
            Ok(success) => {
                log_cleanup(batch_id, item_id, &success.cleanup_failures);
                enqueue_photograph_compensation_cleanup(
                    &self.accounts,
                    item_id,
                    &success.cleanup_failures,
                )
                .await;
                batch
                    .complete_item(
                        item_id,
                        success.value.photograph_id,
                        object_url,
                        thumbnail_url,
                        Utc::now(),
                    )
                    .await;
            }
            Err(MediaWriteError::Upload {
                source,
                compensation_failures,
            }) => {
                log_cleanup(batch_id, item_id, &compensation_failures);
                enqueue_photograph_compensation_cleanup(
                    &self.accounts,
                    item_id,
                    &compensation_failures,
                )
                .await;
                fail(&batch, item_id, "upload", &source).await;
            }
            Err(MediaWriteError::Persistence {
                source,
                compensation_failures,
            }) => {
                log_cleanup(batch_id, item_id, &compensation_failures);
                enqueue_photograph_compensation_cleanup(
                    &self.accounts,
                    item_id,
                    &compensation_failures,
                )
                .await;
                fail(&batch, item_id, "persist", &source).await;
            }
        }
    }
}

struct StagedFileGuard(Option<PathBuf>);
impl StagedFileGuard {
    fn disarm(&mut self) {
        self.0 = None;
    }
}
impl Drop for StagedFileGuard {
    fn drop(&mut self) {
        let Some(path) = self.0.take() else {
            return;
        };
        if let Err(error) = std::fs::remove_file(&path)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            error!(%error, path = %path.display(), "Failed to remove staged batch file during drop cleanup");
        }
    }
}
async fn fail(
    batch: &BatchSession,
    item_id: Uuid,
    operation: &str,
    error: &impl std::fmt::Display,
) {
    error!(%item_id, operation, %error, "Batch media operation failed");
    let reason = format!("{operation} failed: {error}")
        .chars()
        .take(512)
        .collect();
    batch.fail_item(item_id, reason, Utc::now()).await;
}
fn log_cleanup(batch_id: Uuid, item_id: Uuid, failures: &[CleanupFailure]) {
    for failure in failures {
        error!(%batch_id, %item_id, key = %failure.location.key(), error = %failure.error, "Batch media cleanup remains pending");
    }
}
