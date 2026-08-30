use super::error::map_deletion_error;
use crate::{
    dto::responses::{
        photography::delete_photograph_comment_response::DeletePhotographCommentResponse,
        response_data::http_resp,
    },
    errors::code_error::{CodeErrorResp, HandlerResponse},
    features::accounts::domain::role::RoleType,
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

#[utoipa::path(delete, path = "/api/photographs/{photograph_id}/{comment_id}", tag = "photography", params(("photograph_id" = Uuid, Path), ("comment_id" = Uuid, Path)),
responses((status = 200, body = DeletePhotographCommentResponse), (status = 401, body = CodeErrorResp), (status = 404, body = CodeErrorResp)))]
pub async fn delete_photograph_comment(
    Extension(requester_id): Extension<Uuid>,
    Extension(_role_type): Extension<RoleType>,
    State(state): State<Arc<ServerState>>,
    Path((_photograph_id, comment_id)): Path<(Uuid, Uuid)>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    state
        .photography_service()
        .delete_comment(requester_id, comment_id)
        .await
        .map_err(map_deletion_error)?;
    Ok(http_resp(
        DeletePhotographCommentResponse {
            deleted_photograph_comment_id: comment_id,
        },
        (),
        start,
    ))
}
