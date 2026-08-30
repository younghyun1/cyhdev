use std::sync::Arc;

use axum::{Json, extract::State, response::IntoResponse};
use zeroize::Zeroizing;

use crate::{
    dto::{
        requests::auth::reset_password::ResetPasswordProcessRequest,
        responses::{
            auth::reset_password_response::ResetPasswordResponse, response_data::http_resp,
        },
    },
    errors::code_error::{CodeErrorResp, HandlerResponse},
    features::accounts::api::account_error::{AccountMutation, map_account_error},
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
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn reset_password(
    State(state): State<Arc<ServerState>>,
    Json(request): Json<ResetPasswordProcessRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let receipt = state
        .account_service()
        .reset_password(
            request.password_reset_token,
            Zeroizing::new(request.new_password),
        )
        .await
        .map_err(|error| map_account_error(error, AccountMutation::Update))?;

    Ok(http_resp(
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
