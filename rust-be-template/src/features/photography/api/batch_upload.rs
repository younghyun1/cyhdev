use std::{path::PathBuf, sync::Arc};
use axum::{Extension, extract::{Multipart, State}, http::StatusCode, response::IntoResponse};
use serde_derive::Deserialize;
use tracing::{error, warn};
use uuid::Uuid;

use crate::{dto::responses::{photography::batch_status_response::{BatchUploadItem, BatchUploadResponse}, response_data::http_resp},
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err}, features::photography::{
        domain::photograph::PhotographContext, service::batch_upload::{BatchMetadata, MAX_FILE_SIZE_BYTES, MAX_FILES_PER_BATCH, StagedBatchFile}},
    init::state::ServerState, util::{image::batch_pipeline::{append_chunk, batch_temp_dir, open_staging_file},
        media::{image_upload::is_allowed_image_mime, staged_upload::read_bounded_text_field}, time::now::tokio_now}};

const MAX_BATCH_META_BYTES: usize = 256 * 1024;
const MAX_CONTEXT_BYTES: usize = 128;

#[derive(Deserialize)]
struct BatchMetaEntry { comment: Option<String>, lat: Option<f64>, lon: Option<f64> }

#[utoipa::path(post, path = "/api/photographs/batch-upload", tag = "photography", request_body(content_type = "multipart/form-data"),
responses((status = 202, body = BatchUploadResponse), (status = 400, body = CodeErrorResp), (status = 401, body = CodeErrorResp), (status = 403, body = CodeErrorResp), (status = 500, body = CodeErrorResp)))]
pub async fn batch_upload(Extension(user_id): Extension<Uuid>, State(state): State<Arc<ServerState>>, mut multipart: Multipart) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now(); let batch_id = Uuid::now_v7(); let directory = batch_temp_dir(batch_id); let mut guard = StagingGuard::new(directory);
    let mut files = Vec::new(); let mut metadata = None; let mut context = PhotographContext::Photography;
    while let Some(mut field) = multipart.next_field().await.map_err(|error| code_err(CodeError::FILE_UPLOAD_ERROR, error))? {
        match field.name().map(str::to_owned).as_deref() {
            Some("files") | Some("file") => {
                if files.len() >= MAX_FILES_PER_BATCH { return Err(code_err(CodeError::BATCH_TOO_MANY_FILES, format!("maximum {MAX_FILES_PER_BATCH} files per batch"))); }
                let file_name = field.file_name().map(str::to_owned); let content_type = field.content_type().map(str::to_owned);
                if !content_type.as_deref().is_some_and(is_allowed_image_mime) { return Err(code_err(CodeError::FILE_UPLOAD_ERROR, "Unsupported image type in batch")); }
                let item_id = Uuid::now_v7(); let mut file = open_staging_file(batch_id, item_id).await.map_err(|error| code_err(CodeError::FILE_UPLOAD_ERROR, error))?;
                let mut size_bytes = 0_u64;
                loop { let chunk = match field.chunk().await { Ok(Some(chunk)) => chunk, Ok(None) => break, Err(error) => return Err(code_err(CodeError::FILE_UPLOAD_ERROR, error)) };
                    let chunk_size = u64::try_from(chunk.len()).map_err(|_| code_err(CodeError::FILE_UPLOAD_ERROR, "Batch chunk size overflow"))?;
                    size_bytes = size_bytes.checked_add(chunk_size).filter(|size| *size <= MAX_FILE_SIZE_BYTES)
                        .ok_or_else(|| code_err(CodeError::FILE_UPLOAD_ERROR, "A file exceeds the maximum allowed size"))?;
                    append_chunk(&mut file, &chunk).await.map_err(|error| code_err(CodeError::FILE_UPLOAD_ERROR, error))?;
                }
                drop(file); if size_bytes == 0 { return Err(code_err(CodeError::FILE_UPLOAD_ERROR, "Batch files must not be empty")); }
                files.push(StagedBatchFile { item_id, file_name, content_type, size_bytes });
            }
            Some("meta") => {
                let text = read_bounded_text_field(field, MAX_BATCH_META_BYTES).await.map_err(|error| code_err(CodeError::FILE_UPLOAD_ERROR, error))?;
                let parsed = serde_json::from_str::<Vec<BatchMetaEntry>>(&text).map_err(|error| code_err(CodeError::INVALID_REQUEST, error))?;
                metadata = Some(parsed.into_iter().map(|value| BatchMetadata { comment: value.comment, latitude: value.lat, longitude: value.lon }).collect());
            }
            Some("context") | Some("photograph_context") => {
                let text = read_bounded_text_field(field, MAX_CONTEXT_BYTES).await.map_err(|error| code_err(CodeError::FILE_UPLOAD_ERROR, error))?;
                context = PhotographContext::parse(&text).ok_or_else(|| code_err(CodeError::INVALID_REQUEST, "Invalid photograph context"))?;
            }
            Some(other) => warn!(%user_id, field = other, "Ignored unexpected batch field"),
            None => warn!(%user_id, "Ignored unnamed batch field"),
        }
    }
    let metadata = metadata.ok_or_else(|| code_err(CodeError::INVALID_REQUEST, "Missing meta field"))?;
    let accepted = state.photography_service().accept_batch(user_id, batch_id, context, files, metadata).await
        .map_err(super::error::map_photography_error)?;
    guard.disarm();
    Ok((StatusCode::ACCEPTED, http_resp(BatchUploadResponse { batch_id: accepted.batch_id, total: accepted.total,
        items: accepted.items.into_iter().map(|item| BatchUploadItem { item_id: item.item_id, file_name: item.file_name }).collect() }, (), start)))
}

struct StagingGuard(Option<PathBuf>);
impl StagingGuard { fn new(path: PathBuf) -> Self { Self(Some(path)) } fn disarm(&mut self) { self.0 = None; } }
impl Drop for StagingGuard { fn drop(&mut self) { let Some(path) = self.0.take() else { return; }; match tokio::runtime::Handle::try_current() {
    Ok(handle) => { handle.spawn(async move { if let Err(error) = tokio::fs::remove_dir_all(&path).await
        && error.kind() != std::io::ErrorKind::NotFound { warn!(%error, path = %path.display(), "Could not clean rejected batch staging directory"); } }); }
    Err(error) => { if let Err(cleanup_error) = std::fs::remove_dir_all(&path) { error!(%error, %cleanup_error, path = %path.display(), "Could not clean batch staging directory"); } }
} } }
