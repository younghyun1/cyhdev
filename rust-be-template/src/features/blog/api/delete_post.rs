use std::sync::Arc;

use axum::{
    Extension,
    extract::{Path, State},
    response::IntoResponse,
};
use uuid::Uuid;

use crate::{
    dto::responses::{blog::delete_post_response::DeletePostResponse, response_data::http_resp},
    errors::code_error::{CodeErrorResp, HandlerResponse},
    features::accounts::domain::role::RoleType,
    init::state::ServerState,
    util::time::now::tokio_now,
};

use super::error::{BlogOperation, map_blog_error};

#[utoipa::path(
    delete,
    path = "/api/blog/{post_id}",
    tag = "blog",
    params(("post_id" = Uuid, Path, description = "ID of the post to delete")),
    responses(
        (status = 200, description = "Post deleted successfully", body = DeletePostResponse),
        (status = 401, description = "Unauthorized", body = CodeErrorResp),
        (status = 403, description = "Forbidden", body = CodeErrorResp),
        (status = 404, description = "Post not found", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn delete_post(
    Extension(requester_id): Extension<Uuid>,
    Extension(_role): Extension<RoleType>,
    State(state): State<Arc<ServerState>>,
    Path(post_id): Path<Uuid>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    state
        .blog_service()
        .delete_post(requester_id, post_id)
        .await
        .map_err(|error| map_blog_error(error, BlogOperation::Delete))?;
    Ok(http_resp(
        DeletePostResponse {
            deleted_post_id: post_id,
        },
        (),
        start,
    ))
}
