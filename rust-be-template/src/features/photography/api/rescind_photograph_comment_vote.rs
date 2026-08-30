use std::sync::Arc;
use axum::{Extension, extract::{Path, State}, response::IntoResponse};
use uuid::Uuid;
use crate::{dto::responses::response_data::http_resp, errors::code_error::{CodeErrorResp, HandlerResponse}, init::state::ServerState, util::time::now::tokio_now};
use super::error::map_deletion_error;

#[utoipa::path(delete, path = "/api/photographs/{photograph_id}/{comment_id}/vote", tag = "photography", params(("photograph_id" = Uuid, Path), ("comment_id" = Uuid, Path)),
responses((status = 200), (status = 401, body = CodeErrorResp), (status = 404, body = CodeErrorResp)))]
pub async fn rescind_photograph_comment_vote(Extension(user_id): Extension<Uuid>, State(state): State<Arc<ServerState>>, Path((_photograph_id, comment_id)): Path<(Uuid, Uuid)>) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now(); state.photography_service().rescind_comment_vote(user_id, comment_id).await.map_err(map_deletion_error)?;
    Ok(http_resp((), (), start))
}
