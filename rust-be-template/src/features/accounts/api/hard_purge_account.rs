//! Separately authorized hard purge of retained account identity.

use std::sync::Arc;

use axum::{
    Extension,
    extract::{Path, State},
    response::IntoResponse,
};
use uuid::Uuid;

use crate::{
    dto::responses::{
        auth::hard_purge_account_response::{
            HardPurgeAccountResponse, ProfileObjectCleanupFailure,
        },
        response_data::http_resp,
    },
    errors::code_error::{CodeErrorResp, HandlerResponse},
    features::accounts::api::account_error::{AccountMutation, map_account_error},
    init::state::ServerState,
    util::time::now::tokio_now,
};

#[utoipa::path(
    post,
    path = "/api/admin/users/{user_id}/hard-purge",
    tag = "admin",
    params(("user_id" = Uuid, Path, description = "Deleted account identifier")),
    responses(
        (status = 200, description = "Retained identity purged with explicit profile cleanup status", body = HardPurgeAccountResponse),
        (status = 401, description = "Authentication required", body = CodeErrorResp),
        (status = 403, description = "Superuser role required or system actor protected", body = CodeErrorResp),
        (status = 404, description = "Account not found", body = CodeErrorResp),
        (status = 409, description = "Account lifecycle conflict", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn hard_purge_account(
    Extension(requester_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    Path(user_id): Path<Uuid>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let receipt = state
        .account_service()
        .hard_purge_account_with_cleanup(requester_id, user_id)
        .await
        .map_err(|error| map_account_error(error, AccountMutation::Update))?;

    Ok(http_resp(
        HardPurgeAccountResponse {
            user_id: receipt.user_id,
            hard_purged_at: receipt.hard_purged_at,
            profile_objects_deleted: receipt.profile_objects_deleted,
            profile_metadata_deleted: receipt.profile_metadata_deleted,
            profile_cleanup_remaining: receipt.profile_cleanup_remaining,
            profile_cleanup_failures: receipt
                .profile_cleanup_failures
                .into_iter()
                .map(|failure| ProfileObjectCleanupFailure {
                    profile_picture_id: failure.profile_picture_id,
                    object_url: failure.object_url,
                    reason: failure.reason,
                    retryable: failure.retryable,
                })
                .collect(),
        },
        (),
        start,
    ))
}
