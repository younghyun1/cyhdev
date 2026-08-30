use std::sync::Arc;
use axum::{Extension, extract::{Path, State}, response::IntoResponse};
use uuid::Uuid;
use crate::{dto::responses::response_data::http_resp, errors::code_error::{CodeErrorResp, HandlerResponse}, init::state::ServerState, util::time::now::tokio_now};
use super::error::map_deletion_error;

#[utoipa::path(delete, path = "/api/photographs/{photograph_id}/vote", tag = "photography", params(("photograph_id" = Uuid, Path, description = "Photograph to rescind a vote on")),
responses((status = 200, description = "Vote rescinded"), (status = 401, body = CodeErrorResp), (status = 404, body = CodeErrorResp)))]
pub async fn rescind_photograph_vote(Extension(user_id): Extension<Uuid>, State(state): State<Arc<ServerState>>, Path(photograph_id): Path<Uuid>) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now(); state.photography_service().rescind_photograph_vote(user_id, photograph_id).await.map_err(map_deletion_error)?;
    Ok(http_resp((), (), start))
}
