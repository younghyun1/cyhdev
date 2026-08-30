//! `PATCH /api/photographs/{photograph_id}/{comment_id}` — edit a comment.
//! Author or superuser only.

use std::sync::Arc;

use axum::{
    Extension, Json,
    extract::{Path, State},
    response::IntoResponse,
};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    domain::{
        blog::blog::{UserBadgeInfo, VoteState},
        photography::social::{PhotographComment, PhotographCommentResponse},
    },
    dto::{
        requests::photography::update_photograph_comment_request::UpdatePhotographCommentRequest,
        responses::response_data::http_resp,
    },
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    features::accounts::domain::role::RoleType,
    features::accounts::repository::active_user::{
        ActiveUserWriteError, lock_active_superuser, lock_active_user,
    },
    features::accounts::repository::public_authors::load_public_authors,
    init::state::ServerState,
    schema::{photograph_comment_votes, photograph_comments},
    util::time::now::tokio_now,
};

#[utoipa::path(
    patch,
    path = "/api/photographs/{photograph_id}/{comment_id}",
    tag = "photography",
    params(
        ("photograph_id" = Uuid, Path, description = "Photograph id"),
        ("comment_id" = Uuid, Path, description = "Comment to edit")
    ),
    request_body = UpdatePhotographCommentRequest,
    responses(
        (status = 200, description = "Comment updated", body = PhotographCommentResponse),
        (status = 401, description = "Unauthorized", body = CodeErrorResp),
        (status = 404, description = "Comment not found", body = CodeErrorResp)
    )
)]
pub async fn update_photograph_comment(
    Extension(requester_id): Extension<Uuid>,
    Extension(_role_type): Extension<RoleType>,
    State(state): State<Arc<ServerState>>,
    Path((_photograph_id, comment_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdatePhotographCommentRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();

    if request.comment_content.trim().is_empty() {
        return Err(code_err(
            CodeError::INVALID_REQUEST,
            "Comment content must not be empty",
        ));
    }

    let mut conn = state
        .get_conn()
        .await
        .map_err(|e| code_err(CodeError::POOL_ERROR, e))?;

    let (updated, vote_opt): (PhotographComment, Option<bool>) = match conn
        .transaction::<_, ActiveUserWriteError, _>(async |conn| {
            lock_active_user(&mut *conn, requester_id).await?;
            let author_id = photograph_comments::table
                .select(photograph_comments::user_id)
                .filter(photograph_comments::photograph_comment_id.eq(comment_id))
                .first::<Uuid>(&mut *conn)
                .await
                .optional()?
                .ok_or(ActiveUserWriteError::TargetNotFound)?;
            if author_id != requester_id {
                lock_active_superuser(&mut *conn, requester_id).await?;
            }
            let updated = diesel::update(
                photograph_comments::table
                    .filter(photograph_comments::photograph_comment_id.eq(comment_id)),
            )
            .set((
                photograph_comments::photograph_comment_content.eq(&request.comment_content),
                photograph_comments::photograph_comment_updated_at.eq(chrono::Utc::now()),
            ))
            .returning(photograph_comments::all_columns)
            .get_result(&mut *conn)
            .await?;
            let vote_opt = photograph_comment_votes::table
                .filter(photograph_comment_votes::photograph_comment_id.eq(comment_id))
                .filter(photograph_comment_votes::user_id.eq(requester_id))
                .select(photograph_comment_votes::is_upvote)
                .first::<bool>(&mut *conn)
                .await
                .optional()?;
            Ok((updated, vote_opt))
        })
        .await
    {
        Ok(result) => result,
        Err(ActiveUserWriteError::Inactive | ActiveUserWriteError::Denied) => {
            return Err(CodeError::UNAUTHORIZED_ACCESS.into());
        }
        Err(ActiveUserWriteError::TargetNotFound) => {
            return Err(CodeError::COMMENT_NOT_FOUND.into());
        }
        Err(ActiveUserWriteError::Database(e)) => {
            return Err(code_err(CodeError::DB_UPDATE_ERROR, e));
        }
    };
    let vote_state = match vote_opt {
        Some(true) => VoteState::Upvoted,
        Some(false) => VoteState::Downvoted,
        None => VoteState::DidNotVote,
    };

    let author_uid = updated.user_id;
    let public_authors = load_public_authors(&mut conn, &[author_uid])
        .await
        .map_err(|e| code_err(CodeError::DB_QUERY_ERROR, e))?;

    drop(conn);

    let country_map = state.country_map.read().await;
    let (public_user_id, user_badge_info) = match public_authors.get(&author_uid) {
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
    drop(country_map);

    let resp = PhotographCommentResponse::from_comment_votestate_and_badge_info(
        updated,
        vote_state,
        public_user_id,
        user_badge_info,
    );

    Ok(http_resp(resp, (), start))
}
