use std::sync::Arc;

use axum::{
    Extension,
    extract::{Path, State},
    response::IntoResponse,
};
use uuid::Uuid;

use crate::{
    dto::{
        requests::blog::upvote_post_request::UpvotePostRequest,
        responses::{blog::vote_post_response::VotePostResponse, response_data::http_resp},
    },
    errors::code_error::{CodeErrorResp, HandlerResponse},
    init::state::ServerState,
    util::time::now::tokio_now,
};

use super::bounded_json::BlogJson;
use super::error::{BlogOperation, map_blog_error};

#[utoipa::path(
    post,
    path = "/api/blog/{post_id}/vote",
    tag = "blog",
    params(("post_id" = Uuid, Path, description = "ID of the post")),
    request_body = UpvotePostRequest,
    responses(
        (status = 200, description = "Vote recorded", body = VotePostResponse),
        (status = 401, description = "Unauthorized", body = CodeErrorResp),
        (status = 404, description = "Post not found", body = CodeErrorResp),
        (status = 413, description = "Request body too large", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn vote_post(
    Extension(user_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    Path(post_id): Path<Uuid>,
    BlogJson(request): BlogJson<UpvotePostRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let service = state.blog_service();
    let counts = service
        .vote_post(user_id, post_id, request.is_upvote)
        .await
        .map_err(|error| map_blog_error(error, BlogOperation::Insert))?;
    Ok(http_resp(
        VotePostResponse {
            upvote_count: counts.upvotes,
            downvote_count: counts.downvotes,
            is_upvote: request.is_upvote,
        },
        (),
        start,
    ))
}
