use std::sync::Arc;

use axum::{Extension, extract::{Path, State}, response::IntoResponse};
use uuid::Uuid;

use crate::{
    dto::responses::response_data::http_resp,
    errors::code_error::{CodeErrorResp, HandlerResponse},
    init::state::ServerState,
    util::time::now::tokio_now,
};

use super::error::{BlogOperation, map_blog_error};

#[utoipa::path(
    delete,
    path = "/api/blog/{post_id}/{comment_id}/vote",
    tag = "blog",
    params(
        ("post_id" = Uuid, Path, description = "ID of the post"),
        ("comment_id" = Uuid, Path, description = "ID of the comment")
    ),
    responses(
        (status = 200, description = "Vote rescinded successfully"),
        (status = 401, description = "Unauthorized", body = CodeErrorResp),
        (status = 404, description = "Vote does not exist", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn rescind_comment_vote(
    Extension(user_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    Path((_post_id, comment_id)): Path<(Uuid, Uuid)>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    state.blog_service().rescind_comment_vote(user_id, comment_id).await
        .map_err(|error| map_blog_error(error, BlogOperation::VoteRescind))?;
    Ok(http_resp((), (), start))
}
