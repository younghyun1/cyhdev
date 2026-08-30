//! Authenticated full-profile update with current-password confirmation.

use std::sync::Arc;

use axum::{Extension, Json, extract::State, response::IntoResponse};
use uuid::Uuid;

use crate::{
    dto::{
        requests::auth::update_profile_request::UpdateProfileRequest,
        responses::{
            auth::update_profile_response::UpdateProfileResponse, response_data::http_resp,
        },
    },
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    features::accounts::{
        api::account_error::{AccountMutation, map_account_error},
        domain::account::ProfileUpdateCommand,
        error::AccountError,
    },
    init::state::ServerState,
    util::time::now::tokio_now,
};

#[utoipa::path(
    patch,
    path = "/api/auth/profile",
    tag = "auth",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Editable profile fields updated", body = UpdateProfileResponse),
        (status = 400, description = "Invalid profile field or password confirmation", body = CodeErrorResp),
        (status = 401, description = "Authentication required", body = CodeErrorResp),
        (status = 409, description = "Profile identity conflict", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn update_profile(
    Extension(user_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    Json(request): Json<UpdateProfileRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let profile = state
        .account_service()
        .update_profile(
            user_id,
            &request.current_password,
            ProfileUpdateCommand {
                user_name: request.user_name.clone(),
                country: request.user_country,
                language: request.user_language,
                subdivision: request.user_subdivision,
            },
        )
        .await
        .map_err(map_profile_update_error)?;
    Ok(http_resp(
        UpdateProfileResponse {
            user_id: profile.user_id,
            user_name: profile.user_name,
            user_country: profile.country,
            user_language: profile.language,
            user_subdivision: profile.subdivision,
        },
        (),
        start,
    ))
}

fn map_profile_update_error(error: AccountError) -> CodeErrorResp {
    match error {
        AccountError::WrongPassword => code_err(
            CodeError::ACCOUNT_PASSWORD_CONFIRMATION_FAILED,
            AccountError::WrongPassword,
        ),
        error => map_account_error(error, AccountMutation::Update),
    }
}
