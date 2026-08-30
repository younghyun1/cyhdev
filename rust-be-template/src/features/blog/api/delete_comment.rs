use std::sync::Arc;

use axum::{
    Extension,
    extract::{Path, State},
    response::IntoResponse,
};
use uuid::Uuid;

use crate::{
    dto::responses::{
        blog::delete_comment_response::DeleteCommentResponse, response_data::http_resp,
    },
    errors::code_error::{CodeErrorResp, HandlerResponse},
    features::accounts::domain::role::RoleType,
    init::state::ServerState,
    util::time::now::tokio_now,
};

use super::error::{BlogOperation, map_blog_error};

#[utoipa::path(
    delete,
    path = "/api/blog/{post_id}/{comment_id}",
    tag = "blog",
    params(
        ("post_id" = Uuid, Path, description = "ID of the post"),
        ("comment_id" = Uuid, Path, description = "ID of the comment to delete")
    ),
    responses(
        (status = 200, description = "Comment deleted successfully", body = DeleteCommentResponse),
        (status = 401, description = "Unauthorized", body = CodeErrorResp),
        (status = 403, description = "Forbidden", body = CodeErrorResp),
        (status = 404, description = "Comment not found", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn delete_comment(
    Extension(requester_id): Extension<Uuid>,
    Extension(_role): Extension<RoleType>,
    State(state): State<Arc<ServerState>>,
    Path((_post_id, comment_id)): Path<(Uuid, Uuid)>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    state
        .blog_service()
        .delete_comment(requester_id, comment_id)
        .await
        .map_err(|error| map_blog_error(error, BlogOperation::Delete))?;
    Ok(http_resp(
        DeleteCommentResponse {
            deleted_comment_id: comment_id,
        },
        (),
        start,
    ))
}
