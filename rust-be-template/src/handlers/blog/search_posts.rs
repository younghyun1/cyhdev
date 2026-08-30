use std::{collections::HashMap, sync::Arc};

use axum::{
    Extension,
    extract::{Query, State},
    response::IntoResponse,
};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use serde_derive::Deserialize;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    domain::blog::blog::{CachedPostInfo, PostInfoWithVote, UserBadgeInfo, VoteState},
    dto::responses::response_data::http_resp,
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    features::accounts::repository::public_authors::load_public_authors,
    init::state::ServerState,
    routers::middleware::is_logged_in::AuthStatus,
    schema::post_votes,
    util::time::now::tokio_now,
};

#[derive(Deserialize, IntoParams)]
pub struct SearchPostsRequest {
    /// The search query string
    pub q: String,
    /// Search type: "title" for title search, "tag" for tag search
    #[serde(default = "default_search_type")]
    pub search_type: String,
    /// Maximum number of results (default 20, max 100)
    #[serde(default = "default_limit")]
    pub limit: usize,
    /// Page number (1-based)
    #[serde(default = "default_page")]
    pub page: usize,
    /// Optional comma-separated tags to filter by
    pub tags: Option<String>,
}

fn default_search_type() -> String {
    "title".to_string()
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
    Extension(is_logged_in): Extension<AuthStatus>,
    State(state): State<Arc<ServerState>>,
    Query(request): Query<SearchPostsRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();

    let query = request.q.trim();
    let tags = request
        .tags
        .as_deref()
        .unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(|t| t.to_lowercase())
        .collect::<Vec<_>>();

    if query.is_empty() && tags.is_empty() {
        return Err(code_err(
            CodeError::INVALID_REQUEST,
            "Search query cannot be empty",
        ));
    }

    let limit = request.limit.clamp(1, 100);
    let page = request.page.max(1);
    let offset = (page - 1).saturating_mul(limit);
    let search_type = request.search_type.to_lowercase();

    // Perform search based on type
    let (matching_posts, total_matches): (Vec<CachedPostInfo>, usize) = match search_type.as_str() {
        "title" => {
            if !query.is_empty() && !tags.is_empty() {
                state
                    .search_posts_by_title_and_tags(query, &tags, offset, limit)
                    .await
            } else if !query.is_empty() {
                state.search_posts_by_title(query, offset, limit).await
            } else {
                state.search_posts_by_tags(&tags, offset, limit).await
            }
        }
        "tag" => {
            let mut all_tags = tags;
            if !query.is_empty() {
                let normalized = query.to_lowercase();
                if !all_tags.contains(&normalized) {
                    all_tags.push(normalized);
                }
            }
            state.search_posts_by_tags(&all_tags, offset, limit).await
        }
        _ => {
            return Err(code_err(
                CodeError::INVALID_REQUEST,
                "Invalid search_type. Use 'title' or 'tag'",
            ));
        }
    };
    let available_pages = total_matches.div_ceil(limit);

    if matching_posts.is_empty() {
        return Ok(http_resp(
            SearchPostsResponse {
                posts: vec![],
                query: query.to_string(),
                search_type,
                available_pages,
                page,
            },
            (),
            start,
        ));
    }

    // Gather user IDs for author info
    let mut user_ids: Vec<Uuid> = matching_posts.iter().map(|p| p.user_id).collect();
    user_ids.sort();
    user_ids.dedup();

    let post_ids: Vec<Uuid> = matching_posts.iter().map(|p| p.post_id).collect();

    let mut conn = state
        .get_conn()
        .await
        .map_err(|e| code_err(CodeError::POOL_ERROR, e))?;

    let authors = load_public_authors(&mut conn, &user_ids)
        .await
        .map_err(|e| code_err(CodeError::DB_QUERY_ERROR, e))?;

    // Fetch vote states if logged in
    let vote_map = if let AuthStatus::LoggedIn(user_id) = is_logged_in {
        let user_votes: Vec<(Uuid, bool)> = post_votes::table
            .filter(post_votes::post_id.eq_any(&post_ids))
            .filter(post_votes::user_id.eq(user_id))
            .select((post_votes::post_id, post_votes::is_upvote))
            .load::<(Uuid, bool)>(&mut conn)
            .await
            .map_err(|e| code_err(CodeError::DB_QUERY_ERROR, e))?;

        user_votes
            .into_iter()
            .map(|(pid, is_upvote)| {
                let state = if is_upvote {
                    VoteState::Upvoted
                } else {
                    VoteState::Downvoted
                };
                (pid, state)
            })
            .collect::<HashMap<Uuid, VoteState>>()
    } else {
        HashMap::new()
    };

    drop(conn);

    // Get country flag lookup from cache
    let country_map = state.country_map.read().await;

    let posts: Vec<PostInfoWithVote> = matching_posts
        .into_iter()
        .map(|post| {
            let vote_state = vote_map
                .get(&post.post_id)
                .cloned()
                .unwrap_or(VoteState::DidNotVote);

            let (public_user_id, user_badge_info) = match authors.get(&post.user_id) {
                Some(author) => {
                    let country_flag = author
                        .country_code()
                        .and_then(|code| country_map.get_flag_by_code(code));
                    (
                        author.public_user_id(),
                        UserBadgeInfo::from_public_author(author, country_flag),
                    )
                }
                None => (Uuid::nil(), UserBadgeInfo::deleted()),
            };

            PostInfoWithVote::from_cached_info_with_vote(
                post,
                vote_state,
                public_user_id,
                user_badge_info,
            )
        })
        .collect();

    drop(country_map);

    Ok(http_resp(
        SearchPostsResponse {
            posts,
            query: query.to_string(),
            search_type,
            available_pages,
            page,
        },
        (),
        start,
    ))
}
