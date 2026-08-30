//! Bounded multipart streaming into automatically removed temporary files.

use std::path::Path;

use axum::extract::multipart::{Field, MultipartError};
use tempfile::TempPath;
use tokio::io::{AsyncWriteExt, BufWriter};

/// Fixed userspace buffer used while copying multipart chunks to disk.
const UPLOAD_BUFFER_BYTES: usize = 64 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum StageUploadError {
    #[error("multipart stream failed: {0}")]
    Multipart(#[from] MultipartError),
    #[error("temporary upload file failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("upload exceeded its {limit_bytes}-byte limit")]
    TooLarge { limit_bytes: u64 },
    #[error("upload was empty")]
    Empty,
    #[error("multipart text was not UTF-8: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
}

/// A multipart file staged on disk; dropping it removes the file.
pub struct StagedUpload {
    path: TempPath,
    pub file_name: Option<String>,
    pub content_type: Option<String>,
    pub size_bytes: u64,
}

impl StagedUpload {
    pub fn path(&self) -> &Path {
        self.path.as_ref()
    }
}

/// Streams one multipart file field to a bounded temporary file.
pub async fn stage_file_field(
    mut field: Field<'_>,
    max_bytes: u64,
) -> Result<StagedUpload, StageUploadError> {
    let file_name = field.file_name().map(str::to_owned);
    let content_type = field.content_type().map(str::to_owned);
    let named = tempfile::Builder::new()
        .prefix("cyhdev-upload-")
        .suffix(".media")
        .tempfile()?;
    let (file, path) = named.into_parts();
    let mut writer = BufWriter::with_capacity(UPLOAD_BUFFER_BYTES, tokio::fs::File::from_std(file));
    let mut size_bytes = 0_u64;

    while let Some(chunk) = field.chunk().await? {
        let chunk_bytes = u64::try_from(chunk.len()).map_err(|_| StageUploadError::TooLarge {
            limit_bytes: max_bytes,
        })?;
        size_bytes = size_bytes
            .checked_add(chunk_bytes)
            .filter(|total| *total <= max_bytes)
            .ok_or(StageUploadError::TooLarge {
                limit_bytes: max_bytes,
            })?;
        writer.write_all(&chunk).await?;
    }
    writer.flush().await?;
    drop(writer);

    if size_bytes == 0 {
        return Err(StageUploadError::Empty);
    }
    Ok(StagedUpload {
        path,
        file_name,
        content_type,
        size_bytes,
    })
}

/// Reads a small multipart text field without allowing request-sized growth.
pub async fn read_bounded_text_field(
    mut field: Field<'_>,
    max_bytes: usize,
) -> Result<String, StageUploadError> {
    let mut bytes = Vec::with_capacity(max_bytes.min(1024));
    while let Some(chunk) = field.chunk().await? {
        let next_len = bytes
            .len()
            .checked_add(chunk.len())
            .filter(|length| *length <= max_bytes)
            .ok_or(StageUploadError::TooLarge {
                limit_bytes: max_bytes as u64,
            })?;
        bytes.reserve(next_len.saturating_sub(bytes.len()));
        bytes.extend_from_slice(&chunk);
    }
    String::from_utf8(bytes).map_err(StageUploadError::InvalidUtf8)
}
