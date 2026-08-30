//! One photograph batch item's file-backed persistence pipeline.

use std::{path::PathBuf, sync::Arc};

use chrono::Utc;
use diesel_async::{AsyncConnection, RunQueryDsl};
use tracing::{error, warn};
use uuid::Uuid;

use crate::{
    domain::photography::{
        batch::{session::BatchSession, status::ProcessingStatus},
        photographs::{Photograph, PhotographContext, PhotographInsertable},
    },
    features::accounts::repository::active_user::{ActiveUserWriteError, lock_active_superuser},
    init::state::ServerState,
    schema::photographs,
    util::{
        image::{
            batch_pipeline::{BatchPipelineItem, batch_item_path},
            exif_utils::extract_exif_shot_at_from_path,
            map_image_format_to_db_enum::map_image_format_to_str,
            process_uploaded_image_files::process_uploaded_image_files,
            image_variant::{CyhdevImageType, IMAGE_ENCODING_FORMAT},
        },
        media::{
            object_store::{ObjectLocation, S3MediaObjectStore},
            persistence::{
                CleanupFailure, MediaWriteError, MediaWriteSuccess, PendingMediaObject,
                PersistedMedia,
                persist_media_objects,
            },
        },
        s3::AWS_S3_BUCKET_NAME,
    },
};

#[derive(Debug, thiserror::Error)]
enum BatchPersistenceError {
    #[error("database pool checkout failed: {0}")]
    Pool(#[source] anyhow::Error),
    #[error("photograph insert failed: {0}")]
    Insert(#[source] diesel::result::Error),
    #[error("authenticated account is no longer active")]
    Inactive,
}

struct StagedFileGuard {
    path: PathBuf,
}

impl Drop for StagedFileGuard {
    fn drop(&mut self) {
        let path = std::mem::take(&mut self.path);
        tokio::spawn(async move {
            if let Err(error) = tokio::fs::remove_file(&path).await
                && error.kind() != std::io::ErrorKind::NotFound
            {
                warn!(path = %path.display(), error = %error, "Failed to remove staged batch file");
            }
        });
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn process_batch_item(
    state: Arc<ServerState>,
    store: S3MediaObjectStore,
    region: String,
    batch: Arc<BatchSession>,
    batch_id: Uuid,
    item: BatchPipelineItem,
    user_id: Uuid,
    context: PhotographContext,
) {
    let item_id = item.item_id;
    let source_path = batch_item_path(batch_id, item_id);
    let _source_guard = StagedFileGuard {
        path: source_path.clone(),
    };
    batch
        .set_status(item_id, ProcessingStatus::Encoding, Utc::now())
        .await;

    let exif_path = source_path.clone();
    let photograph_shot_at = match tokio::task::spawn_blocking(move || {
        extract_exif_shot_at_from_path(&exif_path)
    })
    .await
    {
        Ok(Ok(value)) => value,
        Ok(Err(source)) => {
            warn!(batch_id = %batch_id, item_id = %item_id, error = %source, "Failed to parse EXIF");
            None
        }
        Err(source) => {
            error!(batch_id = %batch_id, item_id = %item_id, error = %source, "EXIF worker failed");
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
        Err(source) => {
            fail_item(&batch, batch_id, item_id, "encode", &source).await;
            return;
        }
    };
    let main = match outputs.next() {
        Some(output) => output,
        None => {
            fail_item_message(&batch, item_id, "encoder produced no main image").await;
            return;
        }
    };
    let thumbnail = match outputs.next() {
        Some(output) => output,
        None => {
            fail_item_message(&batch, item_id, "encoder produced no thumbnail").await;
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
    let object_url = main_location.public_s3_url(&region);
    let thumbnail_url = thumbnail_location.public_s3_url(&region);
    let pending = [
        PendingMediaObject {
            location: main_location,
            content_type: "image/avif".to_string(),
            source: main.path_buf(),
        },
        PendingMediaObject {
            location: thumbnail_location,
            content_type: "image/avif".to_string(),
            source: thumbnail.path_buf(),
        },
    ];
    batch
        .set_status(item_id, ProcessingStatus::Uploading, Utc::now())
        .await;
    let persistence_batch = Arc::clone(&batch);
    let persisted_object_url = object_url.clone();
    let persisted_thumbnail_url = thumbnail_url.clone();

    let result: Result<
        MediaWriteSuccess<Photograph>,
        MediaWriteError<BatchPersistenceError>,
    > = persist_media_objects(&store, &pending, async move {
        persistence_batch
            .set_status(item_id, ProcessingStatus::Persisting, Utc::now())
            .await;
        let mut connection = state
            .get_conn()
            .await
            .map_err(BatchPersistenceError::Pool)?;
        let photograph: Photograph = connection
            .transaction::<_, ActiveUserWriteError, _>(async |connection| {
                lock_active_superuser(&mut *connection, user_id).await?;
                diesel::insert_into(photographs::table)
                    .values(PhotographInsertable {
                        user_id,
                        photograph_shot_at,
                        photograph_image_type: image_type,
                        photograph_context: context,
                        photograph_is_on_cloud: true,
                        photograph_link: persisted_object_url,
                        photograph_comments: item.comments.clone(),
                        photograph_lat: item.lat,
                        photograph_lon: item.lon,
                        photograph_thumbnail_link: persisted_thumbnail_url,
                    })
                    .get_result(&mut *connection)
                    .await
                    .map_err(ActiveUserWriteError::from)
            })
            .await
            .map_err(|source| match source {
                ActiveUserWriteError::Inactive | ActiveUserWriteError::Denied => {
                    BatchPersistenceError::Inactive
                }
                ActiveUserWriteError::Database(source) => BatchPersistenceError::Insert(source),
                ActiveUserWriteError::TargetNotFound => BatchPersistenceError::Inactive,
            })?;
        drop(connection);
        Ok(PersistedMedia::new(photograph, Vec::new()))
    })
    .await;

    match result {
        Ok(success) => {
            log_cleanup_failures(batch_id, item_id, &success.cleanup_failures);
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
            log_cleanup_failures(batch_id, item_id, &compensation_failures);
            fail_item(&batch, batch_id, item_id, "upload", &source).await;
        }
        Err(MediaWriteError::Persistence {
            source,
            compensation_failures,
        }) => {
            log_cleanup_failures(batch_id, item_id, &compensation_failures);
            fail_item(&batch, batch_id, item_id, "persist", &source).await;
        }
    }
}

async fn fail_item(
    batch: &BatchSession,
    batch_id: Uuid,
    item_id: Uuid,
    operation: &'static str,
    source: &impl std::fmt::Display,
) {
    error!(batch_id = %batch_id, item_id = %item_id, operation, error = %source, "Batch media operation failed");
    fail_item_message(batch, item_id, &format!("{operation} failed: {source}")).await;
}

async fn fail_item_message(batch: &BatchSession, item_id: Uuid, message: &str) {
    batch
        .fail_item(item_id, message.to_string(), Utc::now())
        .await;
}

fn log_cleanup_failures(batch_id: Uuid, item_id: Uuid, failures: &[CleanupFailure]) {
    for failure in failures {
        error!(
            batch_id = %batch_id,
            item_id = %item_id,
            key = %failure.location.key(),
            retryable = failure.is_retryable(),
            error = %failure.error,
            "Batch media cleanup remains pending"
        );
    }
}
