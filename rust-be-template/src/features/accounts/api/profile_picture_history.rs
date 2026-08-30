//! Authenticated profile-picture history management.

use std::sync::Arc;

use axum::{Extension, extract::{Path, State}, response::IntoResponse};
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
        service::profiles::MAX_PROFILE_PICTURE_HISTORY,
    },
    init::state::ServerState,
    util::time::now::tokio_now,
};

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
            maximum_profile_pictures: MAX_PROFILE_PICTURE_HISTORY,
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
    let deletion = state
        .account_service()
        .delete_profile_picture_and_cleanup(
            user_id,
            profile_picture_id,
        )
        .await
        .map_err(map_profile_mutation_error)?
        .ok_or_else(|| {
            code_err(
                CodeError::PROFILE_PICTURE_NOT_FOUND,
                "profile picture is not owned by the current account",
            )
        })?;
    Ok(http_resp(
        DeleteProfilePictureResponse {
            deleted_profile_picture_id: deletion.deleted_profile_picture_id,
            active_profile_picture_id: deletion.active_profile_picture_id,
            cleanup_deleted_count: deletion.cleanup_deleted_count,
            cleanup_failure_count: deletion.cleanup_failure_count,
            cleanup_remaining_count: deletion.cleanup_remaining_count,
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
