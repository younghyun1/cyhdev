//! One batch item: EXIF read, AVIF encode or passthrough, upload, and metadata commit.

use jiff::Timestamp;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tracing::{error, warn};
use uuid::Uuid;

use crate::{
    features::photography::{
        domain::{
            batch::{BatchPipelineItem, ProcessingStatus},
            photograph::{NewPhotograph, Photograph, PhotographContext},
        },
        error::PhotographyError,
        service::batch_session::BatchSession,
    },
    util::{
        image::{
            batch_pipeline::{batch_item_path, batch_thumbnail_source_path},
            exif_utils::extract_exif_shot_at_from_path,
            image_variant::IMAGE_ENCODING_FORMAT,
            map_image_format_to_db_enum::map_image_format_to_str,
        },
        media::{
            object_store::ObjectLocation,
            persistence::{
                CleanupFailure, MediaWriteError, PendingMediaObject, PersistedMedia,
                persist_media_objects,
            },
        },
        s3::AWS_S3_BUCKET_NAME,
    },
};

use super::batch_worker::BatchWorker;
use super::media::enqueue_photograph_compensation_cleanup;
use super::photograph_source::{StagedPhotograph, StagedThumbnailSource, prepare_photograph};

impl BatchWorker {
    pub(super) async fn process(
        &self,
        batch: Arc<BatchSession>,
        directory: &Path,
        item: BatchPipelineItem,
        user_id: Uuid,
        context: PhotographContext,
    ) {
        let batch_id = batch.batch_id;
        let item_id = item.item_id;
        let source_path = batch_item_path(directory, item_id);
        let mut guard = StagedFileGuard(Some(source_path.clone()));
        // Only AVIF items stage a thumbnail source. Removing the batch
        // directory deletes it, so it needs no guard of its own.
        let thumbnail_source_path = item
            .thumbnail_content_type
            .as_ref()
            .map(|_| batch_thumbnail_source_path(directory, item_id));
        self.process_staged(
            batch,
            item,
            user_id,
            context,
            source_path.clone(),
            thumbnail_source_path,
        )
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
        item: BatchPipelineItem,
        user_id: Uuid,
        context: PhotographContext,
        source_path: PathBuf,
        thumbnail_source_path: Option<PathBuf>,
    ) {
        let batch_id = batch.batch_id;
        let item_id = item.item_id;
        batch
            .set_status(item_id, ProcessingStatus::Encoding, Timestamp::now())
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
        // The staged source outlives this call, so an AVIF original can be
        // uploaded straight from it.
        let prepared = match prepare_photograph(StagedPhotograph {
            source: &source_path,
            content_type: item.content_type.as_deref(),
            thumbnail_source: thumbnail_source_path
                .as_deref()
                .map(|path| StagedThumbnailSource {
                    path,
                    content_type: item.thumbnail_content_type.as_deref(),
                }),
        })
        .await
        {
            Ok(prepared) => prepared,
            Err(error) => {
                fail(&batch, item_id, "encode", &error).await;
                return;
            }
        };
        let main = prepared.main;
        let thumbnail = prepared.thumbnail;
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
            .set_status(item_id, ProcessingStatus::Uploading, Timestamp::now())
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
                    .set_status(item_id, ProcessingStatus::Persisting, Timestamp::now())
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
                        Timestamp::now(),
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
    batch.fail_item(item_id, reason, Timestamp::now()).await;
}
fn log_cleanup(batch_id: Uuid, item_id: Uuid, failures: &[CleanupFailure]) {
    for failure in failures {
        error!(%batch_id, %item_id, key = %failure.location.key(), error = %failure.error, "Batch media cleanup remains pending");
    }
}
