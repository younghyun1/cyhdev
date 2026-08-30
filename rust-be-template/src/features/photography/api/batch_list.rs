use super::batch_status::build_batch_status;
use crate::{
    dto::responses::{
        photography::batch_status_response::BatchListResponse, response_data::http_resp,
    },
    errors::code_error::{CodeErrorResp, HandlerResponse},
    init::state::ServerState,
    util::time::now::tokio_now,
};
use axum::{Extension, extract::State, response::IntoResponse};
use std::sync::Arc;
use uuid::Uuid;

#[utoipa::path(get, path = "/api/photographs/batches", tag = "photography", responses((status = 200, body = BatchListResponse), (status = 401, body = CodeErrorResp)))]
pub async fn batch_list(
    Extension(user_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let batches = state.photography_service().owned_batches(user_id).await;
    let mut out = Vec::with_capacity(batches.len());
    for batch in &batches {
        out.push(build_batch_status(batch).await);
    }
    out.sort_by_key(|batch| std::cmp::Reverse(batch.created_at));
    Ok(http_resp(BatchListResponse { batches: out }, (), start))
}
