//! `POST /api/photographs/batch-upload` — accept many photographs at once.
//!
//! Each file part is streamed to a temp file (bounding memory; closes the
//! `upload_photograph.rs` line-52 TODO), aligned by index to a single `meta`
//! JSON sidecar field (`[{comment, lat, lon}, ...]`). The handler mints a batch
//! id, registers an in-memory session, spawns the background pipeline, and
//! replies **202** immediately with the batch id and per-item ids. Status is
//! polled separately via `GET /api/photographs/batch/{batch_id}`.

use std::sync::Arc;

use axum::{
    Extension,
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    domain::photography::{batch::session::BatchSession, photographs::PhotographContext},
    dto::responses::{
        photography::batch_status_response::BatchUploadResponse, response_data::http_resp,
    },
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    init::state::ServerState,
    util::{
        image::batch_pipeline::{append_chunk, batch_temp_dir, open_staging_file, spawn_batch},
        media::staged_upload::read_bounded_text_field,
        time::now::tokio_now,
    },
};

const MAX_BATCH_META_BYTES: usize = 256 * 1024;
const MAX_CONTEXT_BYTES: usize = 128;

use super::batch_upload_support::{
    BatchMetaEntry, MAX_FILE_SIZE_BYTES, MAX_FILES_PER_BATCH, StagedFile, is_allowed_mime_type,
    prepare_batch, remove_staging_dir, StagingDirectoryGuard,
};

#[utoipa::path(
    post,
    path = "/api/photographs/batch-upload",
    tag = "photography",
    request_body(content_type = "multipart/form-data"),
    responses(
        (status = 202, description = "Batch accepted; processing started", body = BatchUploadResponse),
        (status = 400, description = "Invalid batch payload", body = CodeErrorResp),
        (status = 401, description = "Unauthorized", body = CodeErrorResp),
        (status = 403, description = "Forbidden (not superuser)", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn batch_upload(
    Extension(user_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    mut multipart: Multipart,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let batch_id = Uuid::now_v7();
    let dir = batch_temp_dir(batch_id);
    let mut staging_guard = StagingDirectoryGuard::new(dir.clone());

    let mut staged: Vec<StagedFile> = Vec::new();
    let mut meta: Option<Vec<BatchMetaEntry>> = None;
    let mut context = PhotographContext::Photography;

    while let Some(mut field) = multipart.next_field().await.map_err(|e| {
        error!(error = ?e, user_id = %user_id, "Failed to fetch next multipart field");
        code_err(CodeError::FILE_UPLOAD_ERROR, e)
    })? {
        match field.name().map(str::to_owned).as_deref() {
            Some("files") | Some("file") => {
                if staged.len() >= MAX_FILES_PER_BATCH {
                    let _ = tokio::fs::remove_dir_all(&dir).await;
                    warn!(user_id = %user_id, max = MAX_FILES_PER_BATCH, "Batch exceeds maximum file count");
                    return Err(code_err(
                        CodeError::BATCH_TOO_MANY_FILES,
                        format!("maximum {MAX_FILES_PER_BATCH} files per batch"),
                    ));
                }

                let file_name = field.file_name().map(|n| n.to_string());
                let content_type = field.content_type().map(|c| c.to_string());

                if !is_allowed_mime_type(content_type.as_deref()) {
                    let _ = tokio::fs::remove_dir_all(&dir).await;
                    warn!(user_id = %user_id, content_type = ?content_type, "Unsupported image type in batch");
                    return Err(code_err(
                        CodeError::FILE_UPLOAD_ERROR,
                        "Unsupported image type in batch",
                    ));
                }

                let item_id = Uuid::now_v7();
                let mut file = open_staging_file(batch_id, item_id).await.map_err(|e| {
                    error!(error = ?e, user_id = %user_id, "Failed to create staging file");
                    code_err(CodeError::FILE_UPLOAD_ERROR, e)
                })?;

                let mut size_bytes: u64 = 0;
                loop {
                    let chunk = match field.chunk().await {
                        Ok(Some(chunk)) => chunk,
                        Ok(None) => break,
                        Err(e) => {
                            let _ = tokio::fs::remove_dir_all(&dir).await;
                            error!(error = ?e, user_id = %user_id, "Failed reading batch file chunk");
                            return Err(code_err(CodeError::FILE_UPLOAD_ERROR, e));
                        }
                    };
                    size_bytes = match size_bytes.checked_add(chunk.len() as u64) {
                        Some(size) if size <= MAX_FILE_SIZE_BYTES => size,
                        _ => {
                            let _ = tokio::fs::remove_dir_all(&dir).await;
                            warn!(user_id = %user_id, limit = MAX_FILE_SIZE_BYTES, "Batch file exceeds maximum size");
                            return Err(code_err(
                                CodeError::FILE_UPLOAD_ERROR,
                                "A file exceeds the maximum allowed size",
                            ));
                        }
                    };
                    if let Err(e) = append_chunk(&mut file, &chunk).await {
                        let _ = tokio::fs::remove_dir_all(&dir).await;
                        error!(error = ?e, user_id = %user_id, "Failed writing batch file chunk");
                        return Err(code_err(CodeError::FILE_UPLOAD_ERROR, e));
                    }
                }
                drop(file);
                if size_bytes == 0 {
                    let _ = tokio::fs::remove_dir_all(&dir).await;
                    return Err(code_err(
                        CodeError::FILE_UPLOAD_ERROR,
                        "Batch files must not be empty",
                    ));
                }

                staged.push(StagedFile {
                    item_id,
                    file_name,
                    content_type,
                    size_bytes,
                });
            }

            Some("meta") => {
                let text = read_bounded_text_field(field, MAX_BATCH_META_BYTES)
                    .await
                    .map_err(|e| {
                        error!(error = %e, user_id = %user_id, "Failed reading meta field");
                        code_err(CodeError::FILE_UPLOAD_ERROR, e)
                    })?;
                match serde_json::from_str::<Vec<BatchMetaEntry>>(&text) {
                    Ok(parsed) => meta = Some(parsed),
                    Err(e) => {
                        let _ = tokio::fs::remove_dir_all(&dir).await;
                        warn!(error = ?e, user_id = %user_id, "Invalid meta JSON in batch");
                        return Err(code_err(CodeError::INVALID_REQUEST, "Invalid meta JSON"));
                    }
                }
            }

            Some("context") | Some("photograph_context") => {
                let text = read_bounded_text_field(field, MAX_CONTEXT_BYTES)
                    .await
                    .map_err(|e| {
                        error!(error = %e, user_id = %user_id, "Failed reading context field");
                        code_err(CodeError::FILE_UPLOAD_ERROR, e)
                    })?;
                match PhotographContext::from_str(&text) {
                    Some(ctx) => context = ctx,
                    None => {
                        let _ = tokio::fs::remove_dir_all(&dir).await;
                        warn!(user_id = %user_id, value = %text, "Invalid photograph context");
                        return Err(code_err(
                            CodeError::INVALID_REQUEST,
                            "Invalid photograph context",
                        ));
                    }
                }
            }

            Some(other) => {
                warn!(user_id = %user_id, field = other, "Unexpected batch multipart field");
            }
            None => {
                warn!(user_id = %user_id, "Unnamed batch multipart field; ignoring");
            }
        }
    }

    if staged.is_empty() {
        let _ = tokio::fs::remove_dir_all(&dir).await;
        warn!(user_id = %user_id, "Batch contained no files");
        return Err(code_err(CodeError::BATCH_EMPTY, "No files in batch"));
    }

    let meta = match meta {
        Some(meta) => meta,
        None => {
            let _ = tokio::fs::remove_dir_all(&dir).await;
            warn!(user_id = %user_id, "Batch missing meta field");
            return Err(code_err(CodeError::INVALID_REQUEST, "Missing meta field"));
        }
    };

    if meta.len() != staged.len() {
        let _ = tokio::fs::remove_dir_all(&dir).await;
        warn!(
            user_id = %user_id,
            meta_len = meta.len(),
            files_len = staged.len(),
            "Batch meta count does not match file count"
        );
        return Err(code_err(
            CodeError::INVALID_REQUEST,
            "meta count must match file count",
        ));
    }

    let total = staged.len();
    let prepared = match prepare_batch(user_id, context, staged, meta) {
        Ok(prepared) => prepared,
        Err(e) => {
            remove_staging_dir(&dir).await;
            return Err(e);
        }
    };

    let batch = Arc::new(BatchSession::new(
        batch_id,
        user_id,
        total,
        prepared.created_at,
    ));
    for item in prepared.session_items {
        batch.register_item(item).await;
    }
    if !state.register_batch(Arc::clone(&batch)).await {
        let _ = tokio::fs::remove_dir_all(&dir).await;
        warn!(
            user_id = %user_id,
            batch_id = %batch_id,
            "Photograph batch tracker is at capacity"
        );
        return Err(code_err(
            CodeError::INVALID_REQUEST,
            "too many active photograph batches",
        ));
    }

    spawn_batch(
        Arc::clone(&state),
        Arc::clone(&batch),
        prepared.pipeline_items,
        user_id,
        context,
    );
    staging_guard.disarm();

    info!(user_id = %user_id, batch_id = %batch_id, total, "Accepted batch upload; processing started");

    let resp = BatchUploadResponse {
        batch_id,
        total,
        items: prepared.response_items,
    };
    Ok((StatusCode::ACCEPTED, http_resp(resp, (), start)))
}
