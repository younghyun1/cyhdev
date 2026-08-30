use std::sync::Arc;

use axum::{
    Extension, Json,
    extract::{Path, State},
    response::IntoResponse,
};
use diesel::prelude::Insertable;
use uuid::Uuid;

use diesel_async::{AsyncConnection, RunQueryDsl};

use crate::{
    domain::blog::blog::{Comment as DbComment, CommentResponse, UserBadgeInfo, VoteState},
    dto::{
        requests::blog::submit_comment::SubmitCommentRequest, responses::response_data::http_resp,
    },
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    features::accounts::repository::active_user::{ActiveUserWriteError, lock_active_user},
    features::accounts::repository::public_authors::load_public_authors,
    init::state::ServerState,
    routers::middleware::is_logged_in::AuthSession,
    schema::comments,
    util::time::now::tokio_now,
};

// Insert the comment
#[derive(Insertable)]
#[diesel(table_name = comments)]
struct NewComment<'a> {
    pub post_id: &'a Uuid,
    pub user_id: &'a Uuid,
    pub comment_content: &'a str,
    pub parent_comment_id: Option<&'a Uuid>,
}

#[utoipa::path(
    post,
    path = "/api/blog/{post_id}/comment",
    tag = "blog",
    params(
        ("post_id" = Uuid, Path, description = "ID of the post to comment on")
    ),
    request_body = SubmitCommentRequest,
    responses(
        (status = 200, description = "Comment submitted successfully", body = CommentResponse),
        (status = 401, description = "Unauthorized", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn submit_comment(
    Extension(auth_session): Extension<Option<AuthSession>>,
    State(state): State<Arc<ServerState>>,
    Path(post_id): Path<Uuid>,
    Json(request): Json<SubmitCommentRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();

    let mut conn = state
        .get_conn()
        .await
        .map_err(|e| code_err(CodeError::POOL_ERROR, e))?;

    if request.is_guest {
        return Err(CodeError::UNAUTHORIZED_ACCESS.into());
    }

    let auth_session = match auth_session {
        Some(auth_session) => auth_session,
        None => return Err(CodeError::UNAUTHORIZED_ACCESS.into()),
    };
    let user_id = auth_session.user_id;

    let new_comment = NewComment {
        post_id: &post_id,
        user_id: &user_id,
        comment_content: &request.comment_content,
        parent_comment_id: request.parent_comment_id.as_ref(),
    };

    let inserted_comment: DbComment = match conn
        .transaction::<_, ActiveUserWriteError, _>(async |conn| {
            lock_active_user(&mut *conn, user_id).await?;
            diesel::insert_into(comments::table)
                .values(new_comment)
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
        Err(ActiveUserWriteError::Database(e)) => {
            return Err(code_err(CodeError::DB_INSERTION_ERROR, e));
        }
        Err(e) => return Err(code_err(CodeError::DB_INSERTION_ERROR, e)),
    };

    let public_authors = load_public_authors(&mut conn, &[user_id])
        .await
        .map_err(|e| code_err(CodeError::DB_QUERY_ERROR, e))?;

    drop(conn);

    let country_map = state.country_map.read().await;
    let (public_user_id, user_badge_info) = match public_authors.get(&user_id) {
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

    let response = CommentResponse::from_comment_votestate_and_badge_info(
        inserted_comment,
        VoteState::DidNotVote,
        public_user_id,
        user_badge_info,
    );

    Ok(http_resp(response, (), start))
}
