use std::sync::Arc;

use axum::{
    Extension,
    extract::{Query, State},
    response::IntoResponse,
};
use serde_derive::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::{
    dto::responses::response_data::http_resp,
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    features::blog::domain::post::PostInfoWithVote,
    init::state::ServerState,
    routers::middleware::is_logged_in::AuthStatus,
    util::time::now::tokio_now,
};

use super::super::service::queries::{
    BLOG_SEARCH_MAX_LIMIT, BLOG_SEARCH_MAX_OFFSET, BLOG_SEARCH_MAX_QUERY_CHARS,
    BLOG_SEARCH_MAX_TAG_CHARS, BLOG_SEARCH_MAX_TAGS,
};
use super::error::{BlogOperation, map_blog_error};

#[derive(Deserialize, IntoParams)]
pub struct SearchPostsRequest {
    pub q: String,
    #[serde(default = "default_search_type")]
    pub search_type: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default = "default_page")]
    pub page: usize,
    pub tags: Option<String>,
}

fn default_search_type() -> String {
    "title".to_owned()
}
fn default_limit() -> usize {
    20
}
fn default_page() -> usize {
    1
}

#[derive(serde_derive::Serialize, ToSchema)]
pub struct SearchPostsResponse {
    pub posts: Vec<PostInfoWithVote>,
    pub query: String,
    pub search_type: String,
    pub available_pages: usize,
    pub page: usize,
}

#[utoipa::path(
    get,
    path = "/api/blog/search",
    tag = "blog",
    params(SearchPostsRequest),
    responses(
        (status = 200, description = "Search results", body = SearchPostsResponse),
        (status = 400, description = "Invalid search parameters", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn search_posts(
    Extension(auth_status): Extension<AuthStatus>,
    State(state): State<Arc<ServerState>>,
    Query(request): Query<SearchPostsRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let query = request.q.trim();
    if query.chars().count() > BLOG_SEARCH_MAX_QUERY_CHARS {
        return Err(code_err(
            CodeError::INVALID_REQUEST,
            "Search query is too long",
        ));
    }
    let mut tags = Vec::new();
    for tag in request
        .tags
        .as_deref()
        .unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|tag| !tag.is_empty())
    {
        if tags.len() >= BLOG_SEARCH_MAX_TAGS || tag.chars().count() > BLOG_SEARCH_MAX_TAG_CHARS {
            return Err(code_err(
                CodeError::INVALID_REQUEST,
                "Search tags exceed bounded limits",
            ));
        }
        tags.push(tag.to_lowercase());
    }
    if query.is_empty() && tags.is_empty() {
        return Err(code_err(
            CodeError::INVALID_REQUEST,
            "Search query cannot be empty",
        ));
    }
    let limit = request.limit.clamp(1, BLOG_SEARCH_MAX_LIMIT);
    let max_page = (BLOG_SEARCH_MAX_OFFSET / limit).saturating_add(1);
    let page = request.page.clamp(1, max_page);
    let offset = page.saturating_sub(1).saturating_mul(limit);
    let search_type = request.search_type.to_lowercase();
    let service = state.blog_service();
    let search = match search_type.as_str() {
        "title" if !query.is_empty() && !tags.is_empty() => {
            service.search_title_tags(query, &tags, offset, limit).await
        }
        "title" if !query.is_empty() => service.search_title(query, offset, limit).await,
        "title" => service.search_tags(&tags, offset, limit).await,
        "tag" => {
            let mut all_tags = tags;
            if !query.is_empty() {
                let tag = query.to_lowercase();
                if !all_tags.contains(&tag) {
                    all_tags.push(tag);
                }
            }
            service.search_tags(&all_tags, offset, limit).await
        }
        _ => {
            return Err(code_err(
                CodeError::INVALID_REQUEST,
                "Invalid search_type. Use 'title' or 'tag'",
            ));
        }
    };
    let (matching, total) = search.map_err(|error| map_blog_error(error, BlogOperation::Query))?;
    let viewer_id = match auth_status {
        AuthStatus::LoggedIn(user_id) => Some(user_id),
        AuthStatus::LoggedOut => None,
    };
    let posts = service
        .present_posts(matching, viewer_id)
        .await
        .map_err(|error| map_blog_error(error, BlogOperation::Query))?;
    Ok(http_resp(
        SearchPostsResponse {
            posts,
            query: query.to_owned(),
            search_type,
            available_pages: total.div_ceil(limit),
            page,
        },
        (),
        start,
    ))
}
