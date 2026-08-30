//! `GET /api/photographs/{photograph_id}` — public detail endpoint.
//!
//! Increments the naive view count (+1 per call), returns the photograph row
//! (with denormalized view/vote counts), the caller's vote state, and the
//! enriched flat comment list (threaded client-side via parent ids). Mirrors the
//! blog `read_post` enrichment. Public/200 like `read_post`: never 401/403, so
//! the frontend's session guard is not tripped on open.

use std::{collections::HashMap, sync::Arc};

use axum::{
    Extension,
    extract::{Path, State},
    response::IntoResponse,
};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    domain::{
        blog::blog::{UserBadgeInfo, VoteState},
        photography::{
            photographs::Photograph,
            social::{PhotographComment, PhotographCommentResponse},
        },
    },
    dto::responses::{
        photography::read_photograph_response::ReadPhotographResponse, response_data::http_resp,
    },
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    features::accounts::repository::public_authors::load_public_authors,
    init::state::ServerState,
    routers::middleware::is_logged_in::AuthStatus,
    schema::{
        photograph_comment_votes, photograph_comments, photograph_votes, photographs,
    },
    util::time::now::tokio_now,
};

#[utoipa::path(
    get,
    path = "/api/photographs/{photograph_id}",
    tag = "photography",
    params(("photograph_id" = Uuid, Path, description = "Photograph id")),
    responses(
        (status = 200, description = "Photograph detail", body = ReadPhotographResponse),
        (status = 404, description = "Photograph not found", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn read_photograph(
    Extension(is_logged_in): Extension<AuthStatus>,
    State(state): State<Arc<ServerState>>,
    Path(photograph_id): Path<Uuid>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();

    let mut conn = state
        .get_conn()
        .await
        .map_err(|e| code_err(CodeError::POOL_ERROR, e))?;

    // Read the persisted row; the view itself is buffered in RAM and flushed to
    // the DB by a periodic job, so the hot read path does no per-view write.
    let mut photograph: Photograph = photographs::table
        .filter(photographs::photograph_id.eq(photograph_id))
        .select(photographs::all_columns)
        .first::<Photograph>(&mut conn)
        .await
        .optional()
        .map_err(|e| code_err(CodeError::DB_QUERY_ERROR, e))?
        .ok_or_else(|| code_err(CodeError::PHOTOGRAPH_NOT_FOUND, "Photograph not found"))?;
    let photograph_owner_id = photograph.user_id;

    // Record the view in the RAM buffer and present persisted base + pending.
    let pending_views = state.record_view(photograph_id).await;
    photograph.photograph_view_count += pending_views;

    let comments: Vec<PhotographComment> = photograph_comments::table
        .filter(photograph_comments::photograph_id.eq(photograph_id))
        .load::<PhotographComment>(&mut conn)
        .await
        .map_err(|e| code_err(CodeError::DB_QUERY_ERROR, e))?;

    // Author + commenters.
    let mut relevant_user_ids: Vec<Uuid> = comments.iter().map(|c| c.user_id).collect();
    relevant_user_ids.push(photograph.user_id);
    relevant_user_ids.sort();
    relevant_user_ids.dedup();

    let public_authors = load_public_authors(&mut conn, &relevant_user_ids)
        .await
        .map_err(|e| code_err(CodeError::DB_QUERY_ERROR, e))?;

    // Per-comment vote state for the caller.
    let comment_vote_map: HashMap<Uuid, VoteState> =
        if let AuthStatus::LoggedIn(user_id) = is_logged_in {
            let comment_ids: Vec<Uuid> = comments.iter().map(|c| c.photograph_comment_id).collect();
            let user_votes: Vec<(Uuid, bool)> = photograph_comment_votes::table
                .filter(photograph_comment_votes::photograph_comment_id.eq_any(&comment_ids))
                .filter(photograph_comment_votes::user_id.eq(user_id))
                .select((
                    photograph_comment_votes::photograph_comment_id,
                    photograph_comment_votes::is_upvote,
                ))
                .load::<(Uuid, bool)>(&mut conn)
                .await
                .map_err(|e| code_err(CodeError::DB_QUERY_ERROR, e))?;
            user_votes
                .into_iter()
                .map(|(cid, is_upvote)| {
                    let vs = if is_upvote {
                        VoteState::Upvoted
                    } else {
                        VoteState::Downvoted
                    };
                    (cid, vs)
                })
                .collect()
        } else {
            HashMap::new()
        };

    // Caller's vote state for the photograph itself.
    let photograph_vote_state = if let AuthStatus::LoggedIn(user_id) = is_logged_in {
        let opt = photograph_votes::table
            .filter(photograph_votes::photograph_id.eq(photograph_id))
            .filter(photograph_votes::user_id.eq(user_id))
            .select(photograph_votes::is_upvote)
            .first::<bool>(&mut conn)
            .await
            .optional()
            .map_err(|e| code_err(CodeError::DB_QUERY_ERROR, e))?;
        match opt {
            Some(true) => VoteState::Upvoted,
            Some(false) => VoteState::Downvoted,
            None => VoteState::DidNotVote,
        }
    } else {
        VoteState::DidNotVote
    };

    drop(conn);

    let country_map = state.country_map.read().await;

    let presentation_for = |user_id: &Uuid| match public_authors.get(user_id) {
        Some(author) => {
            let country_flag = author
                .country_code()
                .and_then(|code| country_map.get_flag_by_code(code));
            (
                author.public_user_id(),
                UserBadgeInfo::from_public_author(author, country_flag),
                author.is_deleted(),
            )
        }
        None => (Uuid::nil(), UserBadgeInfo::deleted(), true),
    };

    let mut comment_responses: Vec<PhotographCommentResponse> = comments
        .into_iter()
        .map(|comment| {
            let vs = comment_vote_map
                .get(&comment.photograph_comment_id)
                .cloned()
                .unwrap_or(VoteState::DidNotVote);
            let (public_user_id, badge, _) = presentation_for(&comment.user_id);
            PhotographCommentResponse::from_comment_votestate_and_badge_info(
                comment,
                vs,
                public_user_id,
                badge,
            )
        })
        .collect();
    comment_responses.sort_by_key(|c| {
        -(c.photograph_comment_total_upvotes - c.photograph_comment_total_downvotes)
    });

    let (_, author_badge, owner_deleted) = presentation_for(&photograph_owner_id);
    if owner_deleted {
        photograph.anonymize_deleted_owner();
    }
    drop(country_map);

    Ok(http_resp(
        ReadPhotographResponse {
            photograph,
            vote_state: photograph_vote_state,
            comments: comment_responses,
            user_badge_info: author_badge,
        },
        (),
        start,
    ))
}
