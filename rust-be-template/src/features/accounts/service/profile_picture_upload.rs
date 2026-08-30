//! Profile-picture processing, object persistence, and durable cleanup orchestration.

use std::sync::Arc;

use tracing::{error, warn};
use uuid::Uuid;

use crate::{
    features::accounts::{error::AccountError, service::account_service::AccountService},
    util::{
        image::{
            image_variant::{CyhdevImageType, IMAGE_ENCODING_FORMAT},
            map_image_format_to_db_enum::map_image_format_to_str,
            process_uploaded_image_files::process_uploaded_image_files,
        },
        media::{
            cleanup::{REASON_SUPERSEDED_PROFILE_PICTURE, settle_durable_cleanup},
            object_store::{ObjectLocation, ObjectStoreError},
            persistence::{
                CleanupFailure, MediaWriteError, PendingMediaObject, PersistedMedia,
                persist_media_objects,
            },
            staged_upload::StagedUpload,
        },
        s3::AWS_S3_BUCKET_NAME,
    },
};

#[derive(Debug, thiserror::Error)]
pub enum ProfilePictureUploadError {
    #[error("profile-picture processing failed")]
    Processing(#[source] anyhow::Error),
    #[error("profile-picture object upload failed")]
    Upload(#[source] ObjectStoreError),
    #[error("profile-picture metadata persistence failed")]
    Persistence(#[source] AccountError),
}

impl AccountService {
    pub async fn upload_profile_picture(
        self: &Arc<Self>,
        user_id: Uuid,
        upload: StagedUpload,
    ) -> Result<(), ProfilePictureUploadError> {
        let mut processed = process_uploaded_image_files(
            upload.path(),
            None,
            vec![CyhdevImageType::ProfilePicture],
        )
        .await
        .map_err(|source| {
            error!(error = %source, user_id = %user_id, "Failed to process profile picture");
            ProfilePictureUploadError::Processing(source)
        })?
        .into_iter();
        let processed = processed.next().ok_or_else(|| {
            ProfilePictureUploadError::Processing(anyhow::anyhow!(
                "Profile-picture encoder produced no output"
            ))
        })?;
        drop(upload);

        let image_id = Uuid::now_v7();
        let (extension, image_type) = map_image_format_to_str(IMAGE_ENCODING_FORMAT);
        let location =
            ObjectLocation::new(AWS_S3_BUCKET_NAME, format!("images/{image_id}.{extension}"));
        let object_url = location.public_s3_url(&self.media_region);
        let pending = [PendingMediaObject {
            location: location.clone(),
            content_type: "image/avif".to_owned(),
            source: processed.path_buf(),
        }];
        let store = Arc::clone(&self.media_object_store);
        let account_service = Arc::clone(self);
        let persistence_service = Arc::clone(self);
        let result = persist_media_objects(store.as_ref(), &pending, async move {
            let replacement = persistence_service
                .replace_profile_picture_metadata(user_id, image_type, &object_url)
                .await?;
            let superseded = replacement
                .cleanup_objects
                .iter()
                .map(|cleanup| cleanup.location.clone())
                .collect();
            Ok::<_, AccountError>(PersistedMedia::new(replacement, superseded))
        })
        .await;

        match result {
            Ok(success) => {
                if success.value.unresolved_cleanup_count > 0 {
                    warn!(
                        user_id = %user_id,
                        unresolved = success.value.unresolved_cleanup_count,
                        "Superseded legacy profile media requires administrative resolution"
                    );
                }
                settle_durable_cleanup(
                    &account_service,
                    success.value.cleanup_objects,
                    &success.cleaned,
                    &success.cleanup_failures,
                )
                .await;
                log_cleanup_failures(user_id, &success.cleanup_failures);
                Ok(())
            }
            Err(MediaWriteError::Upload {
                source,
                compensation_failures,
            }) => {
                enqueue_compensation_failures(&account_service, image_id, &compensation_failures)
                    .await;
                log_cleanup_failures(user_id, &compensation_failures);
                error!(
                    user_id = %user_id,
                    key = %location.key(),
                    operation = ?source.operation(),
                    retryable = source.is_retryable(),
                    error = %source,
                    "Failed to upload profile picture"
                );
                Err(ProfilePictureUploadError::Upload(source))
            }
            Err(MediaWriteError::Persistence {
                source,
                compensation_failures,
            }) => {
                enqueue_compensation_failures(&account_service, image_id, &compensation_failures)
                    .await;
                log_cleanup_failures(user_id, &compensation_failures);
                error!(error = %source, user_id = %user_id, "Failed to commit profile-picture metadata");
                Err(ProfilePictureUploadError::Persistence(source))
            }
        }
    }
}

async fn enqueue_compensation_failures(
    account_service: &AccountService,
    source_id: Uuid,
    failures: &[CleanupFailure],
) {
    if let Err(error_value) = account_service
        .enqueue_media_cleanup_failures(source_id, REASON_SUPERSEDED_PROFILE_PICTURE, failures)
        .await
    {
        error!(
            error = %error_value,
            source_id = %source_id,
            cleanup_count = failures.len(),
            "Failed to durably enqueue profile compensation cleanup"
        );
    }
}

fn log_cleanup_failures(user_id: Uuid, failures: &[CleanupFailure]) {
    for failure in failures {
        error!(
            user_id = %user_id,
            bucket = %failure.location.bucket(),
            key = %failure.location.key(),
            retryable = failure.is_retryable(),
            error = %failure.error,
            "Media cleanup remains pending"
        );
    }
}
