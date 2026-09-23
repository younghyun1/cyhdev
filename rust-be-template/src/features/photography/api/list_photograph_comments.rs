use super::error::map_photography_error;
use crate::{
    dto::{
        requests::blog::comment_page_query::CommentPageQuery,
        responses::{
            photography::photograph_comment_page_response::PhotographCommentPageResponse,
            response_data::http_resp,
        },
    },
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    features::blog::domain::comment_page::CommentPageRequest,
    init::state::ServerState,
    routers::middleware::is_logged_in::AuthStatus,
    util::time::now::tokio_now,
};
use axum::{
    Extension,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use std::sync::Arc;
use uuid::Uuid;

#[utoipa::path(get, path = "/api/photographs/{photograph_id}/comments", tag = "photography", params(("photograph_id" = Uuid, Path), CommentPageQuery),
responses((status = 200, description = "One oldest-first comment page", body = PhotographCommentPageResponse), (status = 400, body = CodeErrorResp), (status = 404, body = CodeErrorResp), (status = 500, body = CodeErrorResp)))]
pub async fn list_photograph_comments(
    Extension(auth): Extension<AuthStatus>,
    State(state): State<Arc<ServerState>>,
    Path(photograph_id): Path<Uuid>,
    Query(query): Query<CommentPageQuery>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let request =
        CommentPageRequest::parse(query.after_created_at, query.after_comment_id, query.limit)
            .map_err(|error| code_err(CodeError::INVALID_REQUEST, error))?;
    let viewer = match auth {
        AuthStatus::LoggedIn(id) => Some(id),
        AuthStatus::LoggedOut => None,
    };
    let page = state
        .photography_service()
        .comment_page(photograph_id, viewer, request)
        .await
        .map_err(map_photography_error)?;
    Ok(http_resp(
        PhotographCommentPageResponse {
            comments: page.comments,
            next_cursor: page.next_cursor.map(Into::into),
        },
        (),
        start,
    ))
}
