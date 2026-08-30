use std::sync::Arc;

use axum::{Extension, extract::{Path, State}, response::IntoResponse};
use uuid::Uuid;

use crate::{
    dto::{
        requests::blog::upvote_comment_request::UpvoteCommentRequest,
        responses::{blog::vote_comment_response::VoteCommentResponse, response_data::http_resp},
    },
    errors::code_error::{CodeErrorResp, HandlerResponse},
    init::state::ServerState,
    util::time::now::tokio_now,
};

use super::error::{BlogOperation, map_blog_error};
use super::bounded_json::BlogJson;

#[utoipa::path(
    post,
    path = "/api/blog/{post_id}/{comment_id}/vote",
    tag = "blog",
    params(
        ("post_id" = Uuid, Path, description = "ID of the post"),
        ("comment_id" = Uuid, Path, description = "ID of the comment")
    ),
    request_body = UpvoteCommentRequest,
    responses(
        (status = 200, description = "Vote recorded", body = VoteCommentResponse),
        (status = 401, description = "Unauthorized", body = CodeErrorResp),
        (status = 404, description = "Comment not found", body = CodeErrorResp),
        (status = 413, description = "Request body too large", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn vote_comment(
    Extension(user_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    Path((_post_id, comment_id)): Path<(Uuid, Uuid)>,
    BlogJson(request): BlogJson<UpvoteCommentRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let counts = state.blog_service().vote_comment(user_id, comment_id, request.is_upvote).await
        .map_err(|error| map_blog_error(error, BlogOperation::Insert))?;
    Ok(http_resp(VoteCommentResponse {
        upvote_count: counts.upvotes,
        downvote_count: counts.downvotes,
        is_upvote: request.is_upvote,
    }, (), start))
}
