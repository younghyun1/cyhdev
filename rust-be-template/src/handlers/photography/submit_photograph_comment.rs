//! `POST /api/photographs/{photograph_id}/comment` — add a (possibly threaded)
//! comment. Protected tier: any authenticated user.

use std::sync::Arc;

use axum::{
    Extension, Json,
    extract::{Path, State},
    response::IntoResponse,
};
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    domain::{
        blog::blog::{UserBadgeInfo, VoteState},
        photography::social::{NewPhotographComment, PhotographComment, PhotographCommentResponse},
    },
    dto::{
        requests::photography::submit_photograph_comment_request::SubmitPhotographCommentRequest,
        responses::response_data::http_resp,
    },
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    features::accounts::repository::active_user::{ActiveUserWriteError, lock_active_user},
    features::accounts::repository::public_authors::load_public_authors,
    init::state::ServerState,
    schema::photograph_comments,
    util::time::now::tokio_now,
};

#[utoipa::path(
    post,
    path = "/api/photographs/{photograph_id}/comment",
    tag = "photography",
    params(("photograph_id" = Uuid, Path, description = "Photograph to comment on")),
    request_body = SubmitPhotographCommentRequest,
    responses(
        (status = 200, description = "Comment created", body = PhotographCommentResponse),
        (status = 401, description = "Unauthorized", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn submit_photograph_comment(
    Extension(user_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    Path(photograph_id): Path<Uuid>,
    Json(request): Json<SubmitPhotographCommentRequest>,
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

    let new_comment = NewPhotographComment {
        photograph_id: &photograph_id,
        user_id: &user_id,
        photograph_comment_content: &request.comment_content,
        parent_photograph_comment_id: request.parent_comment_id.as_ref(),
    };

    let inserted: PhotographComment = match conn
        .transaction::<_, ActiveUserWriteError, _>(async |conn| {
            lock_active_user(&mut *conn, user_id).await?;
            diesel::insert_into(photograph_comments::table)
                .values(new_comment)
                .returning(photograph_comments::all_columns)
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

    let resp = PhotographCommentResponse::from_comment_votestate_and_badge_info(
        inserted,
        VoteState::DidNotVote,
        public_user_id,
        user_badge_info,
    );

    Ok(http_resp(resp, (), start))
}
