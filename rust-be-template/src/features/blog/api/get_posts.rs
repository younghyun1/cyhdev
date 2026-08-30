use std::sync::Arc;

use axum::{
    Extension,
    extract::{Query, State},
    response::IntoResponse,
};

use crate::{
    dto::{
        requests::blog::get_posts_request::GetPostsRequest,
        responses::{blog::get_posts::GetPostsResponse, response_data::http_resp},
    },
    errors::code_error::{CodeErrorResp, HandlerResponse},
    init::state::ServerState,
    routers::middleware::is_logged_in::AuthStatus,
    util::time::now::tokio_now,
};

use super::error::{BlogOperation, map_blog_error};

#[utoipa::path(
    get,
    path = "/api/blog/posts",
    tag = "blog",
    params(
        ("page" = Option<usize>, Query, description = "Page number"),
        ("posts_per_page" = Option<usize>, Query, description = "Posts per page")
    ),
    responses(
        (status = 200, description = "List of blog posts", body = GetPostsResponse),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn get_posts(
    Extension(auth_status): Extension<AuthStatus>,
    State(state): State<Arc<ServerState>>,
    Query(request): Query<GetPostsRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let viewer_id = match auth_status {
        AuthStatus::LoggedIn(user_id) => Some(user_id),
        AuthStatus::LoggedOut => None,
    };
    let service = state.blog_service();
    let (posts, available_pages) = service
        .list_posts(request.page, request.posts_per_page, viewer_id)
        .await;
    let posts = service
        .present_posts(posts, viewer_id)
        .await
        .map_err(|error| map_blog_error(error, BlogOperation::Query))?;
    Ok(http_resp(
        GetPostsResponse {
            posts,
            available_pages,
        },
        (),
        start,
    ))
}
