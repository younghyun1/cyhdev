use std::sync::Arc;

use axum::{Extension, extract::State, response::IntoResponse};
use uuid::Uuid;

use crate::{
    dto::{
        requests::blog::submit_post_request::SubmitPostRequest,
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
    post,
    path = "/api/blog/posts",
    tag = "blog",
    request_body = SubmitPostRequest,
    responses(
        (status = 200, description = "Post submitted or updated", body = SubmitPostResponse),
        (status = 401, description = "Unauthorized access", body = CodeErrorResp),
        (status = 403, description = "Forbidden access", body = CodeErrorResp),
        (status = 404, description = "Post not found", body = CodeErrorResp),
        (status = 413, description = "Request body too large", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn submit_post(
    Extension(user_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    BlogJson(request): BlogJson<SubmitPostRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let post = state
        .blog_service()
        .save_post(SavePostInput {
            actor_user_id: user_id,
            post_id: request.post_id,
            title: request.post_title,
            markdown: request.post_content,
            tags: request.post_tags,
            published: request.post_is_published,
            owner_required: true,
        })
        .await
        .map_err(|error| map_blog_error(error, BlogOperation::Insert))?;
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
