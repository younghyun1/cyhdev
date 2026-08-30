use std::sync::Arc;

use axum::{
    Extension,
    extract::{Path, State},
    response::IntoResponse,
};
use uuid::Uuid;

use crate::{
    dto::{
        requests::blog::update_post_request::UpdatePostRequest,
        responses::{blog::submit_post_response::SubmitPostResponse, response_data::http_resp},
    },
    errors::code_error::{CodeErrorResp, HandlerResponse},
    features::blog::domain::post::SavePostInput,
    init::state::ServerState,
    util::time::now::tokio_now,
};

use super::bounded_json::BlogJson;
use super::error::{BlogOperation, map_blog_error};

#[utoipa::path(
    patch,
    path = "/api/blog/{post_id}",
    tag = "blog",
    params(("post_id" = Uuid, Path, description = "ID of the post to update")),
    request_body = UpdatePostRequest,
    responses(
        (status = 200, description = "Post updated successfully", body = SubmitPostResponse),
        (status = 401, description = "Unauthorized", body = CodeErrorResp),
        (status = 403, description = "Forbidden", body = CodeErrorResp),
        (status = 404, description = "Post not found", body = CodeErrorResp),
        (status = 413, description = "Request body too large", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn update_post(
    Extension(user_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    Path(post_id): Path<Uuid>,
    BlogJson(request): BlogJson<UpdatePostRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let post = state
        .blog_service()
        .save_post(SavePostInput {
            actor_user_id: user_id,
            post_id: Some(post_id),
            title: request.post_title,
            markdown: request.post_content,
            tags: request.post_tags,
            published: request.post_is_published,
            owner_required: false,
        })
        .await
        .map_err(|error| map_blog_error(error, BlogOperation::Update))?;
    Ok(http_resp(
        SubmitPostResponse {
            post_id: post.post_id,
            post_title: post.post_title,
            post_slug: post.post_slug,
            post_created_at: post.post_created_at,
            post_updated_at: post.post_updated_at,
            post_is_published: post.post_is_published,
        },
        (),
        start,
    ))
}
