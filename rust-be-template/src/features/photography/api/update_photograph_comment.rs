use super::{error::map_update_error, presentation::comment_response};
use crate::{
    dto::{
        requests::photography::update_photograph_comment_request::UpdatePhotographCommentRequest,
        responses::response_data::http_resp,
    },
    errors::code_error::{CodeErrorResp, HandlerResponse},
    features::accounts::domain::role::RoleType,
    features::photography::domain::social::PhotographCommentResponse,
    init::state::ServerState,
    util::time::now::tokio_now,
};
use axum::{
    Extension, Json,
    extract::{Path, State},
    response::IntoResponse,
};
use std::sync::Arc;
use uuid::Uuid;

#[utoipa::path(patch, path = "/api/photographs/{photograph_id}/{comment_id}", tag = "photography", params(("photograph_id" = Uuid, Path), ("comment_id" = Uuid, Path)), request_body = UpdatePhotographCommentRequest,
responses((status = 200, body = PhotographCommentResponse), (status = 401, body = CodeErrorResp), (status = 404, body = CodeErrorResp)))]
pub async fn update_photograph_comment(
    Extension(requester_id): Extension<Uuid>,
    Extension(_role_type): Extension<RoleType>,
    State(state): State<Arc<ServerState>>,
    Path((_photograph_id, comment_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdatePhotographCommentRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let service = state.photography_service();
    let presentation = service
        .update_comment(requester_id, comment_id, request.comment_content)
        .await
        .map_err(map_update_error)?;
    Ok(http_resp(
        comment_response(&service, presentation).await,
        (),
        start,
    ))
}
