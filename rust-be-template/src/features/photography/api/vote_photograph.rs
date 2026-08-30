use super::error::map_insertion_error;
use crate::{
    dto::{
        requests::photography::vote_photograph_request::VotePhotographRequest,
        responses::{
            photography::vote_photograph_response::VotePhotographResponse, response_data::http_resp,
        },
    },
    errors::code_error::{CodeErrorResp, HandlerResponse},
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

#[utoipa::path(post, path = "/api/photographs/{photograph_id}/vote", tag = "photography", params(("photograph_id" = Uuid, Path, description = "Photograph to vote on")), request_body = VotePhotographRequest,
responses((status = 200, description = "Vote recorded", body = VotePhotographResponse), (status = 401, body = CodeErrorResp), (status = 500, body = CodeErrorResp)))]
pub async fn vote_photograph(
    Extension(user_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    Path(photograph_id): Path<Uuid>,
    Json(request): Json<VotePhotographRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let counts = state
        .photography_service()
        .vote_photograph(user_id, photograph_id, request.is_upvote)
        .await
        .map_err(map_insertion_error)?;
    Ok(http_resp(
        VotePhotographResponse {
            upvote_count: counts.upvote_count,
            downvote_count: counts.downvote_count,
            is_upvote: request.is_upvote,
        },
        (),
        start,
    ))
}
