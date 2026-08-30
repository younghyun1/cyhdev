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
    features::accounts::domain::role::RoleType,
    domain::blog::blog::{Comment as DbComment, CommentResponse, UserBadgeInfo, VoteState},
    dto::{
        requests::blog::update_comment_request::UpdateCommentRequest,
        responses::response_data::http_resp,
    },
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    features::accounts::repository::active_user::{
        ActiveUserWriteError, lock_active_superuser, lock_active_user,
    },
    features::accounts::repository::public_authors::load_public_authors,
    init::state::ServerState,
    schema::comments,
    util::time::now::tokio_now,
};

#[utoipa::path(
    patch,
    path = "/api/blog/{post_id}/{comment_id}",
    tag = "blog",
    params(
        ("post_id" = Uuid, Path, description = "ID of the post"),
        ("comment_id" = Uuid, Path, description = "ID of the comment to update")
    ),
    request_body = UpdateCommentRequest,
    responses(
        (status = 200, description = "Comment updated successfully", body = CommentResponse),
        (status = 401, description = "Unauthorized", body = CodeErrorResp),
        (status = 403, description = "Forbidden", body = CodeErrorResp),
        (status = 404, description = "Comment not found", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn update_comment(
    Extension(requester_id): Extension<Uuid>,
    Extension(_role_type): Extension<RoleType>,
    State(state): State<Arc<ServerState>>,
    Path((_post_id, comment_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateCommentRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();

    let mut conn = state
        .get_conn()
        .await
        .map_err(|e| code_err(CodeError::POOL_ERROR, e))?;

    // Check authorship
    let updated_comment: DbComment = match conn
        .transaction::<_, ActiveUserWriteError, _>(async |conn| {
            lock_active_user(&mut *conn, requester_id).await?;
            let author_id = comments::table
                .select(comments::user_id)
                .filter(comments::comment_id.eq(comment_id))
                .first::<Uuid>(&mut *conn)
                .await
                .optional()?
                .ok_or(ActiveUserWriteError::TargetNotFound)?;
            if author_id != requester_id {
                lock_active_superuser(&mut *conn, requester_id).await?;
            }
            diesel::update(comments::table.filter(comments::comment_id.eq(comment_id)))
                .set((
                    comments::comment_content.eq(&request.comment_content),
                    comments::comment_updated_at.eq(chrono::Utc::now()),
                ))
                .returning(comments::all_columns)
                .get_result(&mut *conn)
                .await
                .map_err(ActiveUserWriteError::from)
        })
        .await
    {
        Ok(comment) => comment,
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

    let author_id = updated_comment.user_id;
    let public_authors = load_public_authors(&mut conn, &[author_id])
        .await
        .map_err(|e| code_err(CodeError::DB_QUERY_ERROR, e))?;

    drop(conn);

    let country_map = state.country_map.read().await;
    let (public_user_id, user_badge_info) = match public_authors.get(&author_id) {
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

    Ok(http_resp(
        CommentResponse::from_comment_votestate_and_badge_info(
            updated_comment,
            VoteState::DidNotVote,
            public_user_id,
            user_badge_info,
        ),
        (),
        start,
    ))
}
