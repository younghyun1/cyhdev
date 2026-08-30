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
    features::accounts::domain::role::RoleType,
    dto::responses::{blog::delete_post_response::DeletePostResponse, response_data::http_resp},
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    features::accounts::repository::active_user::{
        ActiveUserWriteError, lock_active_superuser, lock_active_user,
    },
    init::state::ServerState,
    schema::posts,
    util::time::now::tokio_now,
};

#[utoipa::path(
    delete,
    path = "/api/blog/{post_id}",
    tag = "blog",
    params(
        ("post_id" = Uuid, Path, description = "ID of the post to delete")
    ),
    responses(
        (status = 200, description = "Post deleted successfully", body = DeletePostResponse),
        (status = 401, description = "Unauthorized", body = CodeErrorResp),
        (status = 403, description = "Forbidden", body = CodeErrorResp),
        (status = 404, description = "Post not found", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn delete_post(
    Extension(requester_id): Extension<Uuid>,
    Extension(_role_type): Extension<RoleType>,
    State(state): State<Arc<ServerState>>,
    Path(post_id): Path<Uuid>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();

    // 1. Check post author against requester ID.
    let mut conn = state
        .get_conn()
        .await
        .map_err(|e| code_err(CodeError::POOL_ERROR, e))?;

    match conn
        .transaction::<_, ActiveUserWriteError, _>(async |conn| {
            lock_active_user(&mut *conn, requester_id).await?;
            let author_id = posts::table
                .select(posts::user_id)
                .filter(posts::post_id.eq(post_id))
                .first::<Uuid>(&mut *conn)
                .await
                .optional()?
                .ok_or(ActiveUserWriteError::TargetNotFound)?;
            if author_id != requester_id {
                lock_active_superuser(&mut *conn, requester_id).await?;
            }
            diesel::delete(posts::table.filter(posts::post_id.eq(post_id)))
                .execute(&mut *conn)
                .await?;
            Ok(())
        })
        .await
    {
        Ok(()) => tracing::info!(deleted_post_id = %post_id, "Post deleted"),
        Err(ActiveUserWriteError::Inactive | ActiveUserWriteError::Denied) => {
            return Err(CodeError::UNAUTHORIZED_ACCESS.into());
        }
        Err(ActiveUserWriteError::TargetNotFound) => return Err(CodeError::POST_NOT_FOUND.into()),
        Err(ActiveUserWriteError::Database(e)) => {
            return Err(code_err(CodeError::DB_DELETION_ERROR, e));
        }
    }

    drop(conn);

    // delete from state
    state.delete_post_from_cache(post_id).await;

    Ok(http_resp(
        DeletePostResponse {
            deleted_post_id: post_id,
        },
        (),
        start,
    ))
}
