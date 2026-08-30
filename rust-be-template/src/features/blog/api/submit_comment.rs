use std::sync::Arc;

use axum::{Extension, extract::{Path, State}, response::IntoResponse};
use uuid::Uuid;

use crate::{
    features::blog::domain::comment::CommentResponse,
    dto::{requests::blog::submit_comment::SubmitCommentRequest, responses::response_data::http_resp},
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse},
    init::state::ServerState,
    routers::middleware::is_logged_in::AuthSession,
    util::time::now::tokio_now,
};

use super::error::{BlogOperation, map_blog_error};
use super::bounded_json::BlogJson;

#[utoipa::path(
    post,
    path = "/api/blog/{post_id}/comment",
    tag = "blog",
    params(("post_id" = Uuid, Path, description = "ID of the post to comment on")),
    request_body = SubmitCommentRequest,
    responses(
        (status = 200, description = "Comment submitted successfully", body = CommentResponse),
        (status = 400, description = "Invalid comment", body = CodeErrorResp),
        (status = 401, description = "Unauthorized", body = CodeErrorResp),
        (status = 404, description = "Post or parent comment not found", body = CodeErrorResp),
        (status = 413, description = "Request body too large", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn submit_comment(
    Extension(auth_session): Extension<Option<AuthSession>>,
    State(state): State<Arc<ServerState>>,
    Path(post_id): Path<Uuid>,
    BlogJson(request): BlogJson<SubmitCommentRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    if request.is_guest {
        return Err(CodeError::UNAUTHORIZED_ACCESS.into());
    }
    let user_id = match auth_session {
        Some(session) => session.user_id,
        None => return Err(CodeError::UNAUTHORIZED_ACCESS.into()),
    };
    let comment = state.blog_service().submit_comment(
        user_id,
        post_id,
        request.parent_comment_id,
        request.comment_content,
    ).await.map_err(|error| map_blog_error(error, BlogOperation::Insert))?;
    Ok(http_resp(comment, (), start))
}
