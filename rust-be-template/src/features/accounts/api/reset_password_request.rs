use std::sync::Arc;

use axum::{Json, extract::State, response::IntoResponse};

use crate::{
    dto::{
        requests::auth::reset_password_request::ResetPasswordRequest,
        responses::{
            auth::reset_password_request_response::ResetPasswordRequestResponse,
            response_data::http_resp_sensitive,
        },
    },
    errors::code_error::{CodeErrorResp, HandlerResponse},
    features::accounts::{
        api::{
            account_error::{AccountMutation, map_account_error},
            auth_abuse::map_auth_throttle_rejection,
        },
        domain::auth_abuse::{AuthEndpoint, AuthIdentity},
    },
    init::state::ServerState,
    util::time::now::tokio_now,
};

const RESET_REQUEST_RESPONSE_FLOOR: std::time::Duration = std::time::Duration::from_millis(300);

#[utoipa::path(
    post,
    path = "/api/auth/reset-password-request",
    tag = "auth",
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password reset request processed", body = ResetPasswordRequestResponse),
        (status = 400, description = "Invalid email", body = CodeErrorResp),
        (status = 429, description = "Password reset request budget exhausted", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn reset_password_request_process(
    State(state): State<Arc<ServerState>>,
    Json(request): Json<ResetPasswordRequest>,
) -> HandlerResponse<impl IntoResponse> {
    state
        .auth_abuse_service()
        .check_identity(
            AuthEndpoint::PasswordResetRequest,
            AuthIdentity::Email(&request.user_email),
        )
        .await
        .map_err(map_auth_throttle_rejection)?;
    let start = tokio_now();
    state
        .account_service()
        .request_password_reset(&request.user_email)
        .await
        .map_err(|error| map_account_error(error, AccountMutation::Insert))?;
    tokio::time::sleep_until(start + RESET_REQUEST_RESPONSE_FLOOR).await;

    Ok(http_resp_sensitive(
        ResetPasswordRequestResponse {
            message: "If an account matches that email, a password reset link will be sent."
                .to_owned(),
        },
        (),
        start,
    ))
}
