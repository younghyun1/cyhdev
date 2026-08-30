use std::{mem, sync::Arc};

use axum::{Json, extract::State, response::IntoResponse};
use zeroize::Zeroizing;

use crate::{
    dto::{
        requests::auth::reset_password::ResetPasswordProcessRequest,
        responses::{
            auth::reset_password_response::ResetPasswordResponse,
            response_data::http_resp_sensitive,
        },
    },
    errors::code_error::{CodeErrorResp, HandlerResponse},
    features::accounts::{
        api::{account_error::map_password_reset_error, auth_abuse::map_auth_throttle_rejection},
        domain::auth_abuse::{AuthEndpoint, AuthIdentity},
    },
    init::state::ServerState,
    util::time::now::tokio_now,
};

#[utoipa::path(
    post,
    path = "/api/auth/reset-password",
    tag = "auth",
    request_body = ResetPasswordProcessRequest,
    responses(
        (status = 200, description = "Password reset successful", body = ResetPasswordResponse),
        (status = 400, description = "Invalid password or token", body = CodeErrorResp),
        (status = 429, description = "Password reset attempt budget exhausted", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn reset_password(
    State(state): State<Arc<ServerState>>,
    Json(mut request): Json<ResetPasswordProcessRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    state
        .auth_abuse_service()
        .check_identity(
            AuthEndpoint::PasswordResetSubmit,
            AuthIdentity::Token(request.password_reset_token.as_bytes()),
        )
        .await
        .map_err(map_auth_throttle_rejection)?;
    let receipt = state
        .account_service()
        .reset_password(
            request.password_reset_token,
            Zeroizing::new(mem::take(&mut request.new_password)),
        )
        .await
        .map_err(map_password_reset_error)?;

    Ok(http_resp_sensitive(
        ResetPasswordResponse {
            user_id: receipt.user_id,
            user_name: receipt.user_name,
            user_email: receipt.user_email,
            user_updated_at: receipt.updated_at,
        },
        (),
        start,
    ))
}
