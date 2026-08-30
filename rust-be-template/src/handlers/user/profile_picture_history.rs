//! Authenticated profile-picture history management.

use std::sync::Arc;

use axum::{Extension, extract::{Path, State}, response::IntoResponse};
use tracing::error;
use uuid::Uuid;

use crate::{
    dto::responses::{
        response_data::http_resp,
        user::profile_picture_history_response::{
            DeleteProfilePictureResponse, ProfilePictureHistoryItem,
            ProfilePictureHistoryResponse, SelectProfilePictureResponse,
        },
    },
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    features::accounts::{
        error::AccountError,
        repository::profile_pictures::PROFILE_PICTURE_HISTORY_LIMIT,
    },
    init::state::ServerState,
    util::{
        media::{
            cleanup::settle_durable_cleanup,
            object_store::S3MediaObjectStore,
            persistence::cleanup_committed_objects,
        },
        time::now::tokio_now,
    },
};

const PROFILE_CLEANUP_CONCURRENCY: usize = 4;

#[utoipa::path(
    get,
    path = "/api/user/profile-pictures",
    tag = "user",
    responses(
        (status = 200, description = "Bounded profile-picture history", body = ProfilePictureHistoryResponse),
        (status = 401, description = "Authentication required", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn list_profile_pictures(
    Extension(user_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let profile_pictures = state
        .account_service()
        .profile_picture_history(user_id)
        .await
        .map_err(map_profile_query_error)?
        .into_iter()
        .map(Into::into)
        .collect();
    Ok(http_resp(
        ProfilePictureHistoryResponse {
            profile_pictures,
            maximum_profile_pictures: PROFILE_PICTURE_HISTORY_LIMIT as usize,
        },
        (),
        start,
    ))
}

#[utoipa::path(
    post,
    path = "/api/user/profile-pictures/{profile_picture_id}/select",
    tag = "user",
    params(("profile_picture_id" = Uuid, Path, description = "Owned profile-picture id")),
    responses(
        (status = 200, description = "Selected active profile picture", body = SelectProfilePictureResponse),
        (status = 401, description = "Authentication required", body = CodeErrorResp),
        (status = 404, description = "Profile picture not found", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn select_profile_picture(
    Extension(user_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    Path(profile_picture_id): Path<Uuid>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let selected = state
        .account_service()
        .select_profile_picture(user_id, profile_picture_id)
        .await
        .map_err(map_profile_mutation_error)?
        .ok_or_else(|| {
            code_err(
                CodeError::PROFILE_PICTURE_NOT_FOUND,
                "profile picture is not owned by the current account",
            )
        })?;
    Ok(http_resp(
        SelectProfilePictureResponse {
            profile_picture: ProfilePictureHistoryItem::from(selected),
        },
        (),
        start,
    ))
}

#[utoipa::path(
    delete,
    path = "/api/user/profile-pictures/{profile_picture_id}",
    tag = "user",
    params(("profile_picture_id" = Uuid, Path, description = "Owned profile-picture id")),
    responses(
        (status = 200, description = "Deleted profile picture and cleanup status", body = DeleteProfilePictureResponse),
        (status = 401, description = "Authentication required", body = CodeErrorResp),
        (status = 404, description = "Profile picture not found", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn delete_profile_picture(
    Extension(user_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    Path(profile_picture_id): Path<Uuid>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let account_service = state.account_service();
    let deletion = account_service
        .delete_profile_picture(user_id, profile_picture_id)
        .await
        .map_err(map_profile_mutation_error)?
        .ok_or_else(|| {
            code_err(
                CodeError::PROFILE_PICTURE_NOT_FOUND,
                "profile picture is not owned by the current account",
            )
        })?;
    let cleanup_total = deletion.cleanup_objects.len() + deletion.unresolved_cleanup_count;
    let locations = deletion
        .cleanup_objects
        .iter()
        .map(|cleanup| cleanup.location.clone())
        .collect();
    let store = S3MediaObjectStore::from_config(&state.aws_profile_picture_config);
    let (cleaned, failures) = cleanup_committed_objects(
        &store,
        locations,
        PROFILE_CLEANUP_CONCURRENCY,
    )
    .await;
    for failure in &failures {
        error!(
            user_id = %user_id,
            profile_picture_id = %profile_picture_id,
            key = %failure.location.key(),
            retryable = failure.is_retryable(),
            error = %failure.error,
            "Profile-picture object cleanup remains pending"
        );
    }
    let settlement = settle_durable_cleanup(
        &account_service,
        deletion.cleanup_objects,
        &cleaned,
        &failures,
    )
    .await;

    Ok(http_resp(
        DeleteProfilePictureResponse {
            deleted_profile_picture_id: deletion.deleted_profile_picture_id,
            active_profile_picture_id: deletion.active_profile_picture_id,
            cleanup_deleted_count: cleaned.len(),
            cleanup_failure_count: failures.len() + settlement.ledger_errors,
            cleanup_remaining_count: cleanup_total.saturating_sub(settlement.finalized),
        },
        (),
        start,
    ))
}

fn map_profile_query_error(error: AccountError) -> CodeErrorResp {
    let code = match &error {
        AccountError::Pool(_) => CodeError::POOL_ERROR,
        _ => CodeError::DB_QUERY_ERROR,
    };
    code_err(code, error)
}

fn map_profile_mutation_error(error: AccountError) -> CodeErrorResp {
    let code = match &error {
        AccountError::Pool(_) => CodeError::POOL_ERROR,
        _ => CodeError::DB_UPDATE_ERROR,
    };
    code_err(code, error)
}
