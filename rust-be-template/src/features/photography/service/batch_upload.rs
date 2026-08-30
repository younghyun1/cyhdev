use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

use super::photography_service::PhotographyService;
use crate::features::photography::{
    domain::{
        batch::{BatchAccepted, BatchAcceptedItem, BatchItem, BatchPipelineItem, ProcessingStatus},
        photograph::PhotographContext,
    },
    error::PhotographyError,
    service::batch_session::BatchSession,
};

pub const MAX_FILE_SIZE_BYTES: u64 = 150 * 1024 * 1024;
pub const MAX_FILES_PER_BATCH: usize = 50;

pub struct StagedBatchFile {
    pub item_id: Uuid,
    pub file_name: Option<String>,
    pub content_type: Option<String>,
    pub size_bytes: u64,
}

pub struct BatchMetadata {
    pub comment: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

impl PhotographyService {
    pub async fn accept_batch(
        &self,
        user_id: Uuid,
        batch_id: Uuid,
        context: PhotographContext,
        files: Vec<StagedBatchFile>,
        metadata: Vec<BatchMetadata>,
    ) -> Result<BatchAccepted, PhotographyError> {
        if files.is_empty() {
            return Err(PhotographyError::BatchEmpty);
        }
        if files.len() > MAX_FILES_PER_BATCH || files.len() != metadata.len() {
            return Err(PhotographyError::InvalidInput);
        }
        let now = Utc::now();
        let total = files.len();
        let mut pipeline = Vec::with_capacity(total);
        let mut items = Vec::with_capacity(total);
        let mut accepted = Vec::with_capacity(total);
        for (file, metadata) in files.into_iter().zip(metadata) {
            let (comments, latitude, longitude) = resolve_metadata(context, &file, metadata)?;
            accepted.push(BatchAcceptedItem {
                item_id: file.item_id,
                file_name: file.file_name.clone(),
            });
            items.push(BatchItem {
                item_id: file.item_id,
                original_file_name: file.file_name.clone(),
                original_size_bytes: file.size_bytes,
                status: ProcessingStatus::Queued,
                created_at: now,
                updated_at: now,
            });
            pipeline.push(BatchPipelineItem {
                item_id: file.item_id,
                file_name: file.file_name,
                content_type: file.content_type,
                comments,
                latitude,
                longitude,
            });
        }
        let batch = Arc::new(BatchSession::new(batch_id, user_id, total, now));
        for item in items {
            batch.register_item(item).await;
        }
        if !self.register_batch(Arc::clone(&batch)).await {
            return Err(PhotographyError::BatchSaturated);
        }
        self.spawn_batch(batch, pipeline, user_id, context);
        Ok(BatchAccepted {
            batch_id,
            total,
            items: accepted,
        })
    }
}

fn resolve_metadata(
    context: PhotographContext,
    file: &StagedBatchFile,
    metadata: BatchMetadata,
) -> Result<(String, f64, f64), PhotographyError> {
    match context {
        PhotographContext::Photography => {
            let comment = metadata
                .comment
                .filter(|value| !value.trim().is_empty())
                .ok_or(PhotographyError::InvalidInput)?;
            let latitude = coordinate(metadata.latitude, -90.0, 90.0)?
                .ok_or(PhotographyError::InvalidInput)?;
            let longitude = coordinate(metadata.longitude, -180.0, 180.0)?
                .ok_or(PhotographyError::InvalidInput)?;
            Ok((comment, latitude, longitude))
        }
        PhotographContext::Post => {
            let fallback = file
                .file_name
                .clone()
                .unwrap_or_else(|| "post image".to_owned());
            let comment = metadata
                .comment
                .filter(|value| !value.trim().is_empty())
                .unwrap_or(fallback);
            Ok((
                comment,
                coordinate(metadata.latitude, -90.0, 90.0)?.unwrap_or(0.0),
                coordinate(metadata.longitude, -180.0, 180.0)?.unwrap_or(0.0),
            ))
        }
    }
}

fn coordinate(
    value: Option<f64>,
    minimum: f64,
    maximum: f64,
) -> Result<Option<f64>, PhotographyError> {
    match value {
        Some(value) if !value.is_finite() || !(minimum..=maximum).contains(&value) => {
            Err(PhotographyError::InvalidInput)
        }
        value => Ok(value),
    }
}

#[cfg(test)]
mod tests {
    use super::coordinate;

    #[test]
    fn coordinates_reject_nonfinite_and_out_of_range_values() {
        assert!(coordinate(Some(f64::NAN), -90.0, 90.0).is_err());
        assert!(coordinate(Some(181.0), -180.0, 180.0).is_err());
        assert!(matches!(coordinate(None, -90.0, 90.0), Ok(None)));
    }
}
