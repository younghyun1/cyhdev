use crate::{
    dto::responses::{
        photography::batch_status_response::{BatchItemStatus, BatchStatusResponse},
        response_data::http_resp,
    },
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    features::photography::service::batch_session::BatchSession,
    init::state::ServerState,
    util::time::now::tokio_now,
};
use axum::{
    Extension,
    extract::{Path, State},
    response::IntoResponse,
};
use std::sync::Arc;
use uuid::Uuid;

pub async fn build_batch_status(batch: &BatchSession) -> BatchStatusResponse {
    let mut items = batch
        .snapshot_items()
        .await
        .into_iter()
        .map(|item| BatchItemStatus {
            item_id: item.item_id,
            file_name: item.original_file_name,
            original_size_bytes: item.original_size_bytes,
            status: item.status,
            created_at: item.created_at,
            updated_at: item.updated_at,
        })
        .collect::<Vec<_>>();
    items.sort_by(|left, right| {
        left.created_at
            .cmp(&right.created_at)
            .then_with(|| left.item_id.cmp(&right.item_id))
    });
    BatchStatusResponse {
        batch_id: batch.batch_id,
        created_at: batch.created_at,
        total: batch.total,
        completed: batch.completed_count(),
        failed: batch.failed_count(),
        pending: batch.pending_count(),
        done: batch.is_done(),
        items,
    }
}

#[utoipa::path(get, path = "/api/photographs/batch/{batch_id}", tag = "photography", params(("batch_id" = Uuid, Path)),
responses((status = 200, body = BatchStatusResponse), (status = 401, body = CodeErrorResp), (status = 404, body = CodeErrorResp)))]
pub async fn batch_status(
    Extension(user_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    Path(batch_id): Path<Uuid>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let batch = state
        .photography_service()
        .owned_batch(batch_id, user_id)
        .await
        .ok_or_else(|| {
            code_err(
                CodeError::BATCH_NOT_FOUND,
                format!("batch {batch_id} not found for requester"),
            )
        })?;
    Ok(http_resp(build_batch_status(&batch).await, (), start))
}
