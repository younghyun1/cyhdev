//! `DELETE /api/photographs/{photograph_id}/{comment_id}/vote` — remove a
//! comment vote.

use std::sync::Arc;

use axum::{
    Extension,
    extract::{Path, State},
    response::IntoResponse,
};
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl};
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    domain::photography::social::VoteCounts,
    dto::responses::response_data::http_resp,
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    features::accounts::repository::active_user::{ActiveUserWriteError, lock_active_user},
    init::state::ServerState,
    schema::{photograph_comment_votes, photograph_comments},
    util::time::now::tokio_now,
};

#[utoipa::path(
    delete,
    path = "/api/photographs/{photograph_id}/{comment_id}/vote",
    tag = "photography",
    params(
        ("photograph_id" = Uuid, Path, description = "Photograph id"),
        ("comment_id" = Uuid, Path, description = "Comment to rescind a vote on")
    ),
    responses(
        (status = 200, description = "Vote rescinded"),
        (status = 401, description = "Unauthorized", body = CodeErrorResp),
        (status = 404, description = "Vote does not exist", body = CodeErrorResp)
    )
)]
pub async fn rescind_photograph_comment_vote(
    Extension(user_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    Path((_photograph_id, comment_id)): Path<(Uuid, Uuid)>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();

    let mut conn = state
        .get_conn()
        .await
        .map_err(|e| code_err(CodeError::POOL_ERROR, e))?;

    match conn
        .transaction::<_, ActiveUserWriteError, _>(async |conn| {
            lock_active_user(&mut *conn, user_id).await?;
            let affected_rows = diesel::delete(
                photograph_comment_votes::table.filter(
                    photograph_comment_votes::photograph_comment_id
                        .eq(comment_id)
                        .and(photograph_comment_votes::user_id.eq(user_id)),
                ),
            )
            .execute(&mut *conn)
            .await?;

            if affected_rows == 0 {
                return Err(ActiveUserWriteError::TargetNotFound);
            }

            let counts: VoteCounts = diesel::sql_query(
                "SELECT \
                        COUNT(*) FILTER (WHERE is_upvote = true) AS upvote_count, \
                        COUNT(*) FILTER (WHERE is_upvote = false) AS downvote_count \
                     FROM photograph_comment_votes \
                     WHERE photograph_comment_id = $1",
            )
            .bind::<diesel::sql_types::Uuid, Uuid>(comment_id)
            .get_result(&mut *conn)
            .await?;

            diesel::update(
                photograph_comments::table
                    .filter(photograph_comments::photograph_comment_id.eq(comment_id)),
            )
            .set((
                photograph_comments::photograph_comment_total_upvotes.eq(counts.upvote_count),
                photograph_comments::photograph_comment_total_downvotes.eq(counts.downvote_count),
            ))
            .execute(&mut *conn)
            .await?;

            Ok(())
        })
        .await
    {
        Ok(()) => {}
        Err(ActiveUserWriteError::Inactive | ActiveUserWriteError::Denied) => {
            return Err(CodeError::UNAUTHORIZED_ACCESS.into());
        }
        Err(ActiveUserWriteError::TargetNotFound) => {
            return Err(CodeError::UPVOTE_DOES_NOT_EXIST.into());
        }
        Err(ActiveUserWriteError::Database(e)) => {
            return Err(code_err(CodeError::DB_DELETION_ERROR, e));
        }
    }

    Ok(http_resp((), (), start))
}
