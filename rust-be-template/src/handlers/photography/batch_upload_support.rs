use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde_derive::Deserialize;
use tracing::warn;
use uuid::Uuid;

use crate::{
    domain::photography::{
        batch::{session::BatchItem, status::ProcessingStatus},
        photographs::PhotographContext,
    },
    dto::responses::photography::batch_status_response::BatchUploadItem,
    errors::code_error::{CodeError, CodeErrorResp, code_err},
    util::image::batch_pipeline::BatchPipelineItem,
};

pub(super) const MAX_FILE_SIZE_BYTES: u64 = 1024 * 1024 * 150;
pub(super) const MAX_FILES_PER_BATCH: usize = 50;

const ALLOWED_MIME_TYPES: [&str; 16] = [
    "image/png",
    "image/jpeg",
    "image/gif",
    "image/webp",
    "image/x-portable-anymap",
    "image/tiff",
    "image/x-tga",
    "image/vnd-ms.dds",
    "image/bmp",
    "image/vnd.microsoft.icon",
    "image/vnd.radiance",
    "image/x-exr",
    "image/farbfeld",
    "image/avif",
    "image/qoi",
    "image/vnd.zbrush.pcx",
];

#[derive(Debug, Deserialize)]
pub(super) struct BatchMetaEntry {
    pub(super) comment: Option<String>,
    pub(super) lat: Option<f64>,
    pub(super) lon: Option<f64>,
}

pub(super) struct StagedFile {
    pub(super) item_id: Uuid,
    pub(super) file_name: Option<String>,
    pub(super) content_type: Option<String>,
    pub(super) size_bytes: u64,
}

pub(super) struct PreparedBatch {
    pub(super) created_at: DateTime<Utc>,
    pub(super) pipeline_items: Vec<BatchPipelineItem>,
    pub(super) session_items: Vec<BatchItem>,
    pub(super) response_items: Vec<BatchUploadItem>,
}

/// Removes request-owned staging on every early return.
pub(super) struct StagingDirectoryGuard {
    path: Option<PathBuf>,
}

impl StagingDirectoryGuard {
    pub(super) fn new(path: PathBuf) -> Self {
        Self { path: Some(path) }
    }

    pub(super) fn disarm(&mut self) {
        self.path = None;
    }
}

impl Drop for StagingDirectoryGuard {
    fn drop(&mut self) {
        let Some(path) = self.path.take() else {
            return;
        };
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                handle.spawn(async move {
                    if let Err(error) = tokio::fs::remove_dir_all(&path).await
                        && error.kind() != std::io::ErrorKind::NotFound
                    {
                        warn!(path = %path.display(), error = %error, "Failed to remove abandoned batch staging directory");
                    }
                });
            }
            Err(_) => {
                if let Err(error) = std::fs::remove_dir_all(&path)
                    && error.kind() != std::io::ErrorKind::NotFound
                {
                    warn!(path = %path.display(), error = %error, "Failed to synchronously remove abandoned batch staging directory");
                }
            }
        }
    }
}

pub(super) fn is_allowed_mime_type(content_type: Option<&str>) -> bool {
    match content_type {
        Some(content_type) => ALLOWED_MIME_TYPES.contains(&content_type),
        None => false,
    }
}

pub(super) fn prepare_batch(
    user_id: Uuid,
    context: PhotographContext,
    staged: Vec<StagedFile>,
    meta: Vec<BatchMetaEntry>,
) -> Result<PreparedBatch, CodeErrorResp> {
    let created_at = Utc::now();
    let total = staged.len();
    let mut pipeline_items = Vec::with_capacity(total);
    let mut session_items = Vec::with_capacity(total);
    let mut response_items = Vec::with_capacity(total);

    for (file, entry) in staged.into_iter().zip(meta) {
        let (comments, lat, lon) = resolve_metadata(user_id, context, &file, entry)?;
        response_items.push(BatchUploadItem {
            item_id: file.item_id,
            file_name: file.file_name.clone(),
        });
        session_items.push(BatchItem {
            item_id: file.item_id,
            original_file_name: file.file_name.clone(),
            original_size_bytes: file.size_bytes,
            status: ProcessingStatus::Queued,
            created_at,
            updated_at: created_at,
        });
        pipeline_items.push(BatchPipelineItem {
            item_id: file.item_id,
            file_name: file.file_name,
            content_type: file.content_type,
            comments,
            lat,
            lon,
        });
    }

    Ok(PreparedBatch {
        created_at,
        pipeline_items,
        session_items,
        response_items,
    })
}

fn resolve_metadata(
    user_id: Uuid,
    context: PhotographContext,
    file: &StagedFile,
    entry: BatchMetaEntry,
) -> Result<(String, f64, f64), CodeErrorResp> {
    match context {
        PhotographContext::Photography => {
            let comments = required_comment(user_id, entry.comment)?;
            let lat = required_coordinate(user_id, entry.lat, "latitude", -90.0, 90.0)?;
            let lon = required_coordinate(user_id, entry.lon, "longitude", -180.0, 180.0)?;
            Ok((comments, lat, lon))
        }
        PhotographContext::Post => {
            let fallback = match &file.file_name {
                Some(file_name) => file_name.clone(),
                None => "post image".to_string(),
            };
            let comments = match entry.comment {
                Some(comment) if !comment.trim().is_empty() => comment,
                _ => fallback,
            };
            let lat = validate_coordinate(user_id, entry.lat, "latitude", -90.0, 90.0)?
                .unwrap_or(0.0);
            let lon = validate_coordinate(user_id, entry.lon, "longitude", -180.0, 180.0)?
                .unwrap_or(0.0);
            Ok((comments, lat, lon))
        }
    }
}

fn required_comment(user_id: Uuid, comment: Option<String>) -> Result<String, CodeErrorResp> {
    match comment {
        Some(comment) if !comment.trim().is_empty() => Ok(comment),
        _ => {
            warn!(user_id = %user_id, "Batch item missing comment");
            Err(code_err(
                CodeError::INVALID_REQUEST,
                "Each photo requires a comment",
            ))
        }
    }
}

fn required_coordinate(
    user_id: Uuid,
    coordinate: Option<f64>,
    name: &'static str,
    minimum: f64,
    maximum: f64,
) -> Result<f64, CodeErrorResp> {
    match validate_coordinate(user_id, coordinate, name, minimum, maximum)? {
        Some(coordinate) => Ok(coordinate),
        None => {
            warn!(user_id = %user_id, coordinate = name, "Batch item missing coordinate");
            Err(code_err(
                CodeError::INVALID_REQUEST,
                "Each photo requires a location",
            ))
        }
    }
}

fn validate_coordinate(
    user_id: Uuid,
    coordinate: Option<f64>,
    name: &'static str,
    minimum: f64,
    maximum: f64,
) -> Result<Option<f64>, CodeErrorResp> {
    match coordinate {
        Some(coordinate)
            if !coordinate.is_finite() || !(minimum..=maximum).contains(&coordinate) =>
        {
            warn!(user_id = %user_id, coordinate = name, value = coordinate, "Batch coordinate is invalid");
            Err(code_err(
                CodeError::INVALID_REQUEST,
                format!("{name} must be finite and between {minimum} and {maximum}"),
            ))
        }
        value => Ok(value),
    }
}

pub(super) async fn remove_staging_dir(path: &Path) {
    let _ = tokio::fs::remove_dir_all(path).await;
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::validate_coordinate;

    #[test]
    fn rejects_invalid_optional_batch_coordinates() {
        let user_id = Uuid::nil();
        assert!(validate_coordinate(user_id, Some(f64::NAN), "latitude", -90.0, 90.0).is_err());
        assert!(
            validate_coordinate(user_id, Some(181.0), "longitude", -180.0, 180.0).is_err()
        );
        assert!(validate_coordinate(user_id, None, "latitude", -90.0, 90.0).is_ok());
    }
}
