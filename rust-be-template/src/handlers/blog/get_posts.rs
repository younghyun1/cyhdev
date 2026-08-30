use std::{collections::HashMap, sync::Arc};

use crate::{
    domain::blog::blog::{CachedPostInfo, PostInfoWithVote, UserBadgeInfo, VoteState},
    dto::{
        requests::blog::get_posts_request::GetPostsRequest,
        responses::{blog::get_posts::GetPostsResponse, response_data::http_resp},
    },
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    features::accounts::repository::public_authors::load_public_authors,
    init::state::ServerState,
    routers::middleware::is_logged_in::{AuthSession, AuthStatus},
    schema::post_votes,
    util::time::now::tokio_now,
};
use axum::{
    Extension,
    extract::{Query, State},
    response::IntoResponse,
};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

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
    Extension(is_logged_in): Extension<AuthStatus>,
    Extension(auth_session): Extension<Option<AuthSession>>,
    State(state): State<Arc<ServerState>>,
    Query(request): Query<GetPostsRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();

    let include_unpublished = match auth_session {
        Some(auth_session) => auth_session.role_type.is_superuser(),
        None => false,
    };

    let (post_infos, available_pages): (Vec<CachedPostInfo>, usize) = state
        .get_posts_from_cache(request.page, request.posts_per_page, include_unpublished)
        .await;

    let post_ids: Vec<Uuid> = post_infos
        .iter()
        .map(|post| post.post_id)
        .collect::<Vec<Uuid>>();

    let mut user_ids: Vec<Uuid> = post_infos.iter().map(|post| post.user_id).collect();
    user_ids.sort();
    user_ids.dedup();

    let mut conn = state
        .get_conn()
        .await
        .map_err(|e| code_err(CodeError::POOL_ERROR, e))?;

    let authors = load_public_authors(&mut conn, &user_ids)
        .await
        .map_err(|e| code_err(CodeError::DB_QUERY_ERROR, e))?;

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

    let posts: Vec<PostInfoWithVote> = post_infos
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
        GetPostsResponse {
            posts,
            available_pages,
        },
        (),
        start,
    ))
}
