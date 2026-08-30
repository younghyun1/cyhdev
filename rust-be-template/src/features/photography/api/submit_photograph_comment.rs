use std::sync::Arc;
use axum::{Extension, Json, extract::{Path, State}, response::IntoResponse};
use uuid::Uuid;
use crate::{dto::{requests::photography::submit_photograph_comment_request::SubmitPhotographCommentRequest, responses::response_data::http_resp},
    errors::code_error::{CodeErrorResp, HandlerResponse}, features::photography::domain::social::PhotographCommentResponse,
    init::state::ServerState, util::time::now::tokio_now};
use super::{error::map_insertion_error, presentation::comment_response};

#[utoipa::path(post, path = "/api/photographs/{photograph_id}/comment", tag = "photography", params(("photograph_id" = Uuid, Path)), request_body = SubmitPhotographCommentRequest,
responses((status = 200, body = PhotographCommentResponse), (status = 401, body = CodeErrorResp), (status = 500, body = CodeErrorResp)))]
pub async fn submit_photograph_comment(Extension(user_id): Extension<Uuid>, State(state): State<Arc<ServerState>>, Path(photograph_id): Path<Uuid>, Json(request): Json<SubmitPhotographCommentRequest>) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let service = state.photography_service();
    let presentation = service.create_comment(user_id, photograph_id, request.parent_comment_id, request.comment_content).await.map_err(map_insertion_error)?;
    Ok(http_resp(comment_response(&service, presentation).await, (), start))
}
