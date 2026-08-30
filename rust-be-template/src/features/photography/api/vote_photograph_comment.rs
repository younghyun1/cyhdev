use std::sync::Arc;
use axum::{Extension, Json, extract::{Path, State}, response::IntoResponse};
use uuid::Uuid;
use crate::{dto::{requests::photography::vote_photograph_request::VotePhotographRequest, responses::{photography::vote_photograph_response::VotePhotographResponse, response_data::http_resp}},
    errors::code_error::{CodeErrorResp, HandlerResponse}, init::state::ServerState, util::time::now::tokio_now};
use super::error::map_insertion_error;

#[utoipa::path(post, path = "/api/photographs/{photograph_id}/{comment_id}/vote", tag = "photography", params(("photograph_id" = Uuid, Path), ("comment_id" = Uuid, Path)), request_body = VotePhotographRequest,
responses((status = 200, body = VotePhotographResponse), (status = 401, body = CodeErrorResp), (status = 500, body = CodeErrorResp)))]
pub async fn vote_photograph_comment(Extension(user_id): Extension<Uuid>, State(state): State<Arc<ServerState>>, Path((_photograph_id, comment_id)): Path<(Uuid, Uuid)>, Json(request): Json<VotePhotographRequest>) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now(); let counts = state.photography_service().vote_comment(user_id, comment_id, request.is_upvote).await.map_err(map_insertion_error)?;
    Ok(http_resp(VotePhotographResponse { upvote_count: counts.upvote_count, downvote_count: counts.downvote_count, is_upvote: request.is_upvote }, (), start))
}
