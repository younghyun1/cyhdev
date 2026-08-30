use std::sync::Arc;
use tracing::{error, warn};
use uuid::Uuid;

use crate::{
    features::accounts::service::account_service::AccountService,
    features::photography::{
        domain::{
            media::PhotographDeleteReport,
            photograph::{NewPhotograph, Photograph, PhotographContext},
        },
        error::PhotographyError,
    },
    util::{
        image::{
            exif_utils::extract_exif_shot_at_from_path,
            image_variant::{CyhdevImageType, IMAGE_ENCODING_FORMAT},
            map_image_format_to_db_enum::map_image_format_to_str,
            process_uploaded_image_files::process_uploaded_image_files,
        },
        media::{
            cleanup::{
                REASON_DELETED_PHOTOGRAPH_IMAGE, REASON_DELETED_PHOTOGRAPH_THUMBNAIL,
                settle_durable_cleanup,
            },
            object_store::{MediaObjectStore, ObjectLocation},
            persistence::{
                CleanupFailure, MediaWriteError, PendingMediaObject, PersistedMedia,
                cleanup_committed_objects, persist_media_objects,
            },
            staged_upload::StagedUpload,
        },
        s3::AWS_S3_BUCKET_NAME,
    },
};

use super::photography_service::PhotographyService;

const MAX_DELETE_PHOTOGRAPHS: usize = 1_000;
const PHOTOGRAPH_CLEANUP_CONCURRENCY: usize = 8;

pub struct MediaPorts {
    pub(super) object_store: Arc<dyn MediaObjectStore>,
    pub(super) object_store_region: Arc<str>,
    pub(super) accounts: Arc<AccountService>,
}

impl MediaPorts {
    pub fn new(
        object_store: Arc<dyn MediaObjectStore>,
        object_store_region: Arc<str>,
        accounts: Arc<AccountService>,
    ) -> Self {
        Self {
            object_store,
            object_store_region,
            accounts,
        }
    }
}

pub struct PhotographUpload {
    pub source: StagedUpload,
    pub comments: String,
    pub latitude: f64,
    pub longitude: f64,
    pub context: PhotographContext,
}

impl PhotographyService {
    pub async fn upload_photograph(
        &self,
        user_id: Uuid,
        upload: PhotographUpload,
    ) -> Result<Photograph, PhotographyError> {
        let source_path = upload.source.path().to_path_buf();
        let shot_at = read_exif(source_path.clone(), user_id).await;
        let mut outputs = process_uploaded_image_files(
            &source_path,
            None,
            vec![CyhdevImageType::Photograph, CyhdevImageType::Thumbnail],
        )
        .await
        .map_err(PhotographyError::Image)?
        .into_iter();
        let main = outputs.next().ok_or_else(|| {
            PhotographyError::Image(anyhow::anyhow!("encoder produced no main image"))
        })?;
        let thumbnail = outputs.next().ok_or_else(|| {
            PhotographyError::Image(anyhow::anyhow!("encoder produced no thumbnail"))
        })?;
        drop(source_path);
        drop(upload.source);

        let image_id = Uuid::now_v7();
        let (extension, image_type) = map_image_format_to_str(IMAGE_ENCODING_FORMAT);
        let image_location =
            ObjectLocation::new(AWS_S3_BUCKET_NAME, format!("images/{image_id}.{extension}"));
        let thumbnail_location = ObjectLocation::new(
            AWS_S3_BUCKET_NAME,
            format!("thumbnails/{image_id}.{extension}"),
        );
        let object_url = image_location.public_s3_url(&self.media.object_store_region);
        let thumbnail_url = thumbnail_location.public_s3_url(&self.media.object_store_region);
        let pending = [
            PendingMediaObject {
                location: image_location,
                content_type: "image/avif".to_owned(),
                source: main.path_buf(),
            },
            PendingMediaObject {
                location: thumbnail_location,
                content_type: "image/avif".to_owned(),
                source: thumbnail.path_buf(),
            },
        ];
        let context = upload.context;
        let comments = upload.comments;
        let latitude = upload.latitude;
        let longitude = upload.longitude;
        let object_url_for_insert = object_url.clone();
        let thumbnail_url_for_insert = thumbnail_url.clone();
        let result = persist_media_objects(self.media.object_store.as_ref(), &pending, async {
            let photograph = self
                .repository
                .insert_photograph(
                    user_id,
                    NewPhotograph {
                        user_id,
                        photograph_shot_at: shot_at,
                        photograph_image_type: image_type,
                        photograph_context: context,
                        photograph_is_on_cloud: true,
                        photograph_link: object_url_for_insert,
                        photograph_comments: comments,
                        photograph_lat: latitude,
                        photograph_lon: longitude,
                        photograph_thumbnail_link: thumbnail_url_for_insert,
                    },
                )
                .await?;
            Ok(PersistedMedia::new(photograph, Vec::new()))
        })
        .await;
        let mut photograph = match result {
            Ok(success) => {
                log_cleanup(&success.cleanup_failures);
                enqueue_photograph_compensation_cleanup(
                    &self.media.accounts,
                    image_id,
                    &success.cleanup_failures,
                )
                .await;
                success.value
            }
            Err(MediaWriteError::Upload {
                source,
                compensation_failures,
            }) => {
                log_cleanup(&compensation_failures);
                enqueue_photograph_compensation_cleanup(
                    &self.media.accounts,
                    image_id,
                    &compensation_failures,
                )
                .await;
                return Err(PhotographyError::Media(anyhow::Error::new(source)));
            }
            Err(MediaWriteError::Persistence {
                source,
                compensation_failures,
            }) => {
                log_cleanup(&compensation_failures);
                enqueue_photograph_compensation_cleanup(
                    &self.media.accounts,
                    image_id,
                    &compensation_failures,
                )
                .await;
                return Err(source);
            }
        };
        match self.repository.user_is_deleted(user_id).await {
            Ok(true) => photograph.anonymize_deleted_owner(),
            Ok(false) => {}
            Err(error) => {
                warn!(%user_id, %error, "Committed photograph owner recheck failed; returning a privacy-safe owner projection");
                photograph.anonymize_deleted_owner();
            }
        }
        Ok(photograph)
    }

    pub async fn delete_photographs(
        &self,
        requester_id: Uuid,
        mut ids: Vec<Uuid>,
    ) -> Result<PhotographDeleteReport, PhotographyError> {
        normalize_ids(&mut ids)?;
        if ids.is_empty() {
            return Ok(PhotographDeleteReport {
                deleted_count: 0,
                s3_deleted_count: 0,
                cleanup_failure_count: 0,
                cleanup_remaining_count: 0,
                unresolved_cleanup_count: 0,
            });
        }
        let retired = self
            .repository
            .retire_photographs(requester_id, &ids)
            .await?;
        let cleanup_total = retired.cleanup.resolved.len() + retired.cleanup.unresolved_count;
        let locations = retired
            .cleanup
            .resolved
            .iter()
            .map(|cleanup| cleanup.location.clone())
            .collect();
        let (cleaned, failures) = cleanup_committed_objects(
            self.media.object_store.as_ref(),
            locations,
            PHOTOGRAPH_CLEANUP_CONCURRENCY,
        )
        .await;
        log_cleanup(&failures);
        let settlement = settle_durable_cleanup(
            &self.media.accounts,
            retired.cleanup.resolved,
            &cleaned,
            &failures,
        )
        .await;
        Ok(PhotographDeleteReport {
            deleted_count: retired.deleted_rows,
            s3_deleted_count: cleaned.len(),
            cleanup_failure_count: failures.len() + settlement.ledger_errors,
            cleanup_remaining_count: cleanup_total.saturating_sub(settlement.finalized),
            unresolved_cleanup_count: retired.cleanup.unresolved_count,
        })
    }
}

async fn read_exif(
    path: std::path::PathBuf,
    user_id: Uuid,
) -> Option<chrono::DateTime<chrono::Utc>> {
    match tokio::task::spawn_blocking(move || extract_exif_shot_at_from_path(&path)).await {
        Ok(Ok(value)) => value,
        Ok(Err(error)) => {
            warn!(%user_id, %error, "Failed to parse photograph EXIF");
            None
        }
        Err(error) => {
            error!(%user_id, %error, "Photograph EXIF worker failed");
            None
        }
    }
}

fn log_cleanup(failures: &[CleanupFailure]) {
    for failure in failures {
        error!(bucket = %failure.location.bucket(), key = %failure.location.key(), retryable = failure.is_retryable(), error = %failure.error, "Photograph media cleanup remains pending");
    }
}

pub(super) async fn enqueue_photograph_compensation_cleanup(
    accounts: &AccountService,
    source_id: Uuid,
    failures: &[CleanupFailure],
) {
    for failure in failures {
        let reason = photograph_cleanup_reason(&failure.location);
        match accounts
            .enqueue_media_cleanup_failures(source_id, reason, std::slice::from_ref(failure))
            .await
        {
            Ok(report) => {
                warn!(%source_id, reason, submitted = report.submitted, inserted = report.inserted,
                already_registered = report.already_registered, "Registered photograph compensation failure for durable retry")
            }
            Err(error) => {
                error!(%source_id, reason, %error, bucket = %failure.location.bucket(), key = %failure.location.key(),
                "Failed to register photograph compensation failure in the durable cleanup ledger")
            }
        }
    }
}

fn photograph_cleanup_reason(location: &ObjectLocation) -> &'static str {
    if location.key().starts_with("thumbnails/") {
        REASON_DELETED_PHOTOGRAPH_THUMBNAIL
    } else {
        REASON_DELETED_PHOTOGRAPH_IMAGE
    }
}

fn normalize_ids(ids: &mut Vec<Uuid>) -> Result<(), PhotographyError> {
    ids.sort_unstable();
    ids.dedup();
    if ids.len() > MAX_DELETE_PHOTOGRAPHS {
        Err(PhotographyError::InvalidInput)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_DELETE_PHOTOGRAPHS, normalize_ids, photograph_cleanup_reason};
    use crate::util::media::{
        cleanup::{REASON_DELETED_PHOTOGRAPH_IMAGE, REASON_DELETED_PHOTOGRAPH_THUMBNAIL},
        object_store::ObjectLocation,
    };
    use uuid::Uuid;
    #[test]
    fn deletion_deduplicates_before_cap() {
        let mut ids = vec![Uuid::nil(); MAX_DELETE_PHOTOGRAPHS + 1];
        assert!(normalize_ids(&mut ids).is_ok());
        assert_eq!(ids.len(), 1);
    }
    #[test]
    fn deletion_rejects_distinct_overflow() {
        let mut ids = (0..=MAX_DELETE_PHOTOGRAPHS)
            .map(|value| Uuid::from_u128(value as u128))
            .collect();
        assert!(normalize_ids(&mut ids).is_err());
    }
    #[test]
    fn compensation_cleanup_preserves_photograph_object_role() {
        let image = ObjectLocation::new("bucket", "images/source.avif");
        let thumbnail = ObjectLocation::new("bucket", "thumbnails/source.avif");
        assert_eq!(
            photograph_cleanup_reason(&image),
            REASON_DELETED_PHOTOGRAPH_IMAGE
        );
        assert_eq!(
            photograph_cleanup_reason(&thumbnail),
            REASON_DELETED_PHOTOGRAPH_THUMBNAIL
        );
    }
}
