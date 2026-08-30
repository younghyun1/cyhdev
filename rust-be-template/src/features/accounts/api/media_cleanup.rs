//! Superuser inspection and reconciliation of unresolved media cleanup records.

use std::sync::Arc;

use axum::{
    Extension, Json,
    extract::{Path, State},
    response::IntoResponse,
};
use uuid::Uuid;

use crate::{
    dto::{
        requests::admin::media_cleanup_request::ResolveMediaCleanupRequest,
        responses::{
            admin::media_cleanup_response::{
                ResolveMediaCleanupResponse, UnresolvedMediaCleanupItem,
                UnresolvedMediaCleanupResponse,
            },
            response_data::http_resp,
        },
    },
    errors::code_error::{CodeErrorResp, HandlerResponse},
    features::accounts::api::account_error::{AccountMutation, map_account_error},
    init::state::ServerState,
    util::time::now::tokio_now,
};

#[utoipa::path(
    get,
    path = "/api/admin/media-cleanup/unresolved",
    tag = "admin",
    responses(
        (status = 200, description = "Bounded unresolved media cleanup records", body = UnresolvedMediaCleanupResponse),
        (status = 401, description = "Authentication required", body = CodeErrorResp),
        (status = 403, description = "Current database role is not superuser", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn unresolved_media_cleanup(
    Extension(requester_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let records = state
        .account_service()
        .unresolved_media_cleanup(requester_id)
        .await
        .map_err(|error| map_account_error(error, AccountMutation::Update))?
        .into_iter()
        .map(|record| UnresolvedMediaCleanupItem {
            cleanup_id: record.cleanup_id,
            original_url: record.original_url,
            reason: record.reason,
            source_id: record.source_id,
            created_at: record.created_at,
        })
        .collect();
    Ok(http_resp(
        UnresolvedMediaCleanupResponse { records },
        (),
        start,
    ))
}

#[utoipa::path(
    post,
    path = "/api/admin/media-cleanup/{cleanup_id}/resolve",
    tag = "admin",
    params(("cleanup_id" = Uuid, Path, description = "Cleanup record identifier")),
    request_body = ResolveMediaCleanupRequest,
    responses(
        (status = 200, description = "Cleanup object address reconciled", body = ResolveMediaCleanupResponse),
        (status = 400, description = "Invalid object address", body = CodeErrorResp),
        (status = 401, description = "Authentication required", body = CodeErrorResp),
        (status = 403, description = "Current database role is not superuser", body = CodeErrorResp),
        (status = 404, description = "Cleanup record not found", body = CodeErrorResp),
        (status = 409, description = "Stored cleanup state conflicts with reconciliation", body = CodeErrorResp),
        (status = 413, description = "Reconciliation request exceeds the private JSON limit", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn resolve_media_cleanup(
    Extension(requester_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    Path(cleanup_id): Path<Uuid>,
    Json(request): Json<ResolveMediaCleanupRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let resolution = state
        .account_service()
        .resolve_media_cleanup(
            requester_id,
            cleanup_id,
            &request.expected_original_url,
            &request.bucket,
            &request.key,
        )
        .await
        .map_err(|error| map_account_error(error, AccountMutation::Update))?;
    Ok(http_resp(
        ResolveMediaCleanupResponse {
            cleanup_id: resolution.cleanup_id,
            bucket: resolution.bucket,
            key: resolution.key,
            original_url: resolution.original_url,
        },
        (),
        start,
    ))
}
