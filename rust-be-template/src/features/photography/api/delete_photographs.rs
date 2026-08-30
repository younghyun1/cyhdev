use super::error::map_deletion_error;
use crate::{
    dto::{
        requests::photography::delete_photographs_request::DeletePhotographsRequest,
        responses::{
            photography::delete_photographs_response::DeletePhotographsResponse,
            response_data::http_resp,
        },
    },
    errors::code_error::{CodeErrorResp, HandlerResponse},
    init::state::ServerState,
    util::time::now::tokio_now,
};
use axum::{Extension, Json, extract::State, response::IntoResponse};
use std::sync::Arc;
use uuid::Uuid;

#[utoipa::path(delete, path = "/api/photographs/delete", tag = "photography", request_body = DeletePhotographsRequest,
responses((status = 200, body = DeletePhotographsResponse), (status = 400, body = CodeErrorResp), (status = 401, body = CodeErrorResp), (status = 403, body = CodeErrorResp), (status = 500, body = CodeErrorResp)))]
pub async fn delete_photographs(
    Extension(user_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    Json(request): Json<DeletePhotographsRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let report = state
        .photography_service()
        .delete_photographs(user_id, request.photograph_ids)
        .await
        .map_err(map_deletion_error)?;
    Ok(http_resp(
        DeletePhotographsResponse {
            deleted_count: report.deleted_count,
            s3_deleted_count: report.s3_deleted_count,
            cleanup_failure_count: report.cleanup_failure_count,
            cleanup_remaining_count: report.cleanup_remaining_count,
            unresolved_cleanup_count: report.unresolved_cleanup_count,
        },
        (),
        start,
    ))
}
