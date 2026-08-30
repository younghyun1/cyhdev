//! `DELETE /api/photographs/{photograph_id}/{comment_id}` — hard-delete a
//! comment. Author or superuser only. FK `ON DELETE CASCADE` removes child
//! comments and the comment's votes.

use std::sync::Arc;

use axum::{
    Extension,
    extract::{Path, State},
    response::IntoResponse,
};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    dto::responses::{
        photography::delete_photograph_comment_response::DeletePhotographCommentResponse,
        response_data::http_resp,
    },
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    features::accounts::domain::role::RoleType,
    features::accounts::repository::active_user::{
        ActiveUserWriteError, lock_active_superuser, lock_active_user,
    },
    init::state::ServerState,
    schema::photograph_comments,
    util::time::now::tokio_now,
};

#[utoipa::path(
    delete,
    path = "/api/photographs/{photograph_id}/{comment_id}",
    tag = "photography",
    params(
        ("photograph_id" = Uuid, Path, description = "Photograph id"),
        ("comment_id" = Uuid, Path, description = "Comment to delete")
    ),
    responses(
        (status = 200, description = "Comment deleted", body = DeletePhotographCommentResponse),
        (status = 401, description = "Unauthorized", body = CodeErrorResp),
        (status = 404, description = "Comment not found", body = CodeErrorResp)
    )
)]
pub async fn delete_photograph_comment(
    Extension(requester_id): Extension<Uuid>,
    Extension(_role_type): Extension<RoleType>,
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
            diesel::delete(
                photograph_comments::table
                    .filter(photograph_comments::photograph_comment_id.eq(comment_id)),
            )
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
            return Err(CodeError::COMMENT_NOT_FOUND.into());
        }
        Err(ActiveUserWriteError::Database(e)) => {
            return Err(code_err(CodeError::DB_DELETION_ERROR, e));
        }
    }

    drop(conn);

    Ok(http_resp(
        DeletePhotographCommentResponse {
            deleted_photograph_comment_id: comment_id,
        },
        (),
        start,
    ))
}
