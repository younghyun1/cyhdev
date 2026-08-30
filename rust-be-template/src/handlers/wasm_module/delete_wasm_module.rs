use std::sync::Arc;

use axum::{
    Extension,
    extract::{Path, State},
    response::IntoResponse,
};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::{AsyncConnection, RunQueryDsl};
use serde_derive::Serialize;
use tracing::{error, info};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    dto::responses::response_data::http_resp,
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    features::accounts::repository::active_user::{ActiveUserWriteError, lock_active_superuser},
    init::state::ServerState,
    schema::wasm_module,
    util::{
        media::{
            cleanup::{
                MediaCleanupRequest, REASON_DELETED_WASM_THUMBNAIL,
                enqueue_media_cleanup, settle_durable_cleanup,
            },
            object_store::S3MediaObjectStore,
            persistence::cleanup_committed_objects,
        },
        time::now::tokio_now,
    },
};

#[derive(Debug, Serialize, ToSchema)]
pub struct DeleteWasmModuleResponse {
    pub deleted_wasm_module_id: Uuid,
    pub cleanup_deleted_count: usize,
    pub cleanup_failure_count: usize,
    pub cleanup_remaining_count: usize,
    pub unresolved_cleanup_count: usize,
}

const WASM_CLEANUP_CONCURRENCY: usize = 4;

/// DELETE /api/wasm-modules/{wasm_module_id}
/// Superuser only - deletes a WASM module (DB record and cache)
#[utoipa::path(
    delete,
    path = "/api/wasm-modules/{wasm_module_id}",
    tag = "wasm_module",
    params(
        ("wasm_module_id" = Uuid, Path, description = "WASM module UUID")
    ),
    responses(
        (status = 200, description = "WASM module deleted", body = DeleteWasmModuleResponse),
        (status = 401, description = "Unauthorized", body = CodeErrorResp),
        (status = 403, description = "Forbidden (not superuser)", body = CodeErrorResp),
        (status = 404, description = "WASM module not found", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn delete_wasm_module(
    Extension(user_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    Path(wasm_module_id): Path<Uuid>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();

    let mut conn = state.get_conn().await.map_err(|e| {
        error!(error = ?e, "Failed to get DB connection");
        code_err(CodeError::POOL_ERROR, e)
    })?;

    let cleanup = match conn
        .transaction::<_, ActiveUserWriteError, _>(async |conn| {
            lock_active_superuser(&mut *conn, user_id).await?;
            let thumbnail_url = wasm_module::table
                .filter(wasm_module::wasm_module_id.eq(wasm_module_id))
                .select(wasm_module::wasm_module_thumbnail_link)
                .for_update()
                .first::<String>(&mut *conn)
                .await
                .optional()?
                .ok_or(ActiveUserWriteError::TargetNotFound)?;
            let cleanup = enqueue_media_cleanup(
                conn,
                vec![MediaCleanupRequest {
                    original_url: thumbnail_url,
                    reason: REASON_DELETED_WASM_THUMBNAIL,
                    source_id: wasm_module_id,
                }],
            )
            .await?;
            let deleted = diesel::delete(
                wasm_module::table.filter(wasm_module::wasm_module_id.eq(wasm_module_id)),
            )
            .execute(&mut *conn)
            .await?;
            if deleted == 0 {
                return Err(ActiveUserWriteError::TargetNotFound);
            }
            Ok(cleanup)
        })
        .await
    {
        Ok(cleanup) => cleanup,
        Err(ActiveUserWriteError::Inactive | ActiveUserWriteError::Denied) => {
            return Err(CodeError::UNAUTHORIZED_ACCESS.into());
        }
        Err(ActiveUserWriteError::TargetNotFound) => {
            return Err(code_err(CodeError::DB_QUERY_ERROR, "WASM module not found"));
        }
        Err(ActiveUserWriteError::Database(e)) => {
            error!(error = ?e, wasm_module_id = %wasm_module_id, "Failed to delete WASM module from DB");
            return Err(code_err(CodeError::DB_DELETION_ERROR, e));
        }
    };

    drop(conn);

    // Remove from cache
    state.invalidate_wasm_module(wasm_module_id).await;

    let cleanup_total = cleanup.resolved.len() + cleanup.unresolved_count;
    let locations = cleanup
        .resolved
        .iter()
        .map(|cleanup| cleanup.location.clone())
        .collect();
    let store = S3MediaObjectStore::from_config(&state.aws_profile_picture_config);
    let (cleaned, failures) =
        cleanup_committed_objects(&store, locations, WASM_CLEANUP_CONCURRENCY).await;
    let settlement = settle_durable_cleanup(
        &state.account_service(),
        cleanup.resolved,
        &cleaned,
        &failures,
    )
    .await;
    for failure in &failures {
        error!(
            wasm_module_id = %wasm_module_id,
            key = %failure.location.key(),
            retryable = failure.is_retryable(),
            error = %failure.error,
            "WASM thumbnail cleanup remains pending"
        );
    }

    info!(
        wasm_module_id = %wasm_module_id,
        user_id = %user_id,
        "WASM module deleted"
    );

    Ok(http_resp(
        DeleteWasmModuleResponse {
            deleted_wasm_module_id: wasm_module_id,
            cleanup_deleted_count: cleaned.len(),
            cleanup_failure_count: failures.len() + settlement.ledger_errors,
            cleanup_remaining_count: cleanup_total.saturating_sub(settlement.finalized),
            unresolved_cleanup_count: cleanup.unresolved_count,
        },
        (),
        start,
    ))
}
