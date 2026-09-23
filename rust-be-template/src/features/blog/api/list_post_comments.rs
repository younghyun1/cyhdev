use std::sync::Arc;

use axum::{
    Extension,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use uuid::Uuid;

use crate::{
    dto::{
        requests::blog::comment_page_query::CommentPageQuery,
        responses::{blog::comment_page_response::CommentPageResponse, response_data::http_resp},
    },
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    features::blog::domain::comment_page::CommentPageRequest,
    init::state::ServerState,
    routers::middleware::is_logged_in::AuthStatus,
    util::time::now::tokio_now,
};

use super::error::{BlogOperation, map_blog_error};

#[utoipa::path(
    get,
    path = "/api/blog/posts/{post_id}/comments",
    tag = "blog",
    params(("post_id" = Uuid, Path, description = "Post UUID"), CommentPageQuery),
    responses(
        (status = 200, description = "One oldest-first comment page", body = CommentPageResponse),
        (status = 400, description = "Invalid cursor or page size", body = CodeErrorResp),
        (status = 404, description = "Post not found", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn list_post_comments(
    Extension(auth_status): Extension<AuthStatus>,
    State(state): State<Arc<ServerState>>,
    Path(post_id): Path<Uuid>,
    Query(query): Query<CommentPageQuery>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let request =
        CommentPageRequest::parse(query.after_created_at, query.after_comment_id, query.limit)
            .map_err(|error| code_err(CodeError::INVALID_REQUEST, error))?;
    let viewer_id = match auth_status {
        AuthStatus::LoggedIn(user_id) => Some(user_id),
        AuthStatus::LoggedOut => None,
    };
    let page = state
        .blog_service()
        .comment_page(post_id, viewer_id, request)
        .await
        .map_err(|error| map_blog_error(error, BlogOperation::Query))?;
    Ok(http_resp(
        CommentPageResponse {
            comments: page.comments,
            next_cursor: page.next_cursor.map(Into::into),
        },
        (),
        start,
    ))
}
