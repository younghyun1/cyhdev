use std::sync::Arc;

use axum::{
    Extension,
    extract::{Path, State},
    response::IntoResponse,
};
use uuid::Uuid;

use crate::{
    dto::{
        requests::blog::update_comment_request::UpdateCommentRequest,
        responses::response_data::http_resp,
    },
    errors::code_error::{CodeErrorResp, HandlerResponse},
    features::accounts::domain::role::RoleType,
    features::blog::domain::comment::CommentResponse,
    init::state::ServerState,
    util::time::now::tokio_now,
};

use super::bounded_json::BlogJson;
use super::error::{BlogOperation, map_blog_error};

#[utoipa::path(
    patch,
    path = "/api/blog/{post_id}/{comment_id}",
    tag = "blog",
    params(
        ("post_id" = Uuid, Path, description = "ID of the post"),
        ("comment_id" = Uuid, Path, description = "ID of the comment to update")
    ),
    request_body = UpdateCommentRequest,
    responses(
        (status = 200, description = "Comment updated successfully", body = CommentResponse),
        (status = 401, description = "Unauthorized", body = CodeErrorResp),
        (status = 403, description = "Forbidden", body = CodeErrorResp),
        (status = 404, description = "Comment not found", body = CodeErrorResp),
        (status = 413, description = "Request body too large", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn update_comment(
    Extension(requester_id): Extension<Uuid>,
    Extension(_role): Extension<RoleType>,
    State(state): State<Arc<ServerState>>,
    Path((_post_id, comment_id)): Path<(Uuid, Uuid)>,
    BlogJson(request): BlogJson<UpdateCommentRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let comment = state
        .blog_service()
        .update_comment(requester_id, comment_id, request.comment_content)
        .await
        .map_err(|error| map_blog_error(error, BlogOperation::Update))?;
    Ok(http_resp(comment, (), start))
}
