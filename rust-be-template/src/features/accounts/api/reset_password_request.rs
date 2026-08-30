use std::sync::Arc;

use axum::{Json, extract::State, response::IntoResponse};

use crate::{
    dto::{
        requests::auth::reset_password_request::ResetPasswordRequest,
        responses::{
            auth::reset_password_request_response::ResetPasswordRequestResponse,
            response_data::http_resp,
        },
    },
    errors::code_error::{CodeErrorResp, HandlerResponse},
    features::accounts::api::account_error::{AccountMutation, map_account_error},
    init::state::ServerState,
    util::time::now::tokio_now,
};

#[utoipa::path(
    post,
    path = "/api/auth/reset-password-request",
    tag = "auth",
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password reset request processed", body = ResetPasswordRequestResponse),
        (status = 400, description = "Invalid email", body = CodeErrorResp),
        (status = 404, description = "User not found", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn reset_password_request_process(
    State(state): State<Arc<ServerState>>,
    Json(request): Json<ResetPasswordRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let receipt = state
        .account_service()
        .request_password_reset(&request.user_email)
        .await
        .map_err(|error| map_account_error(error, AccountMutation::Insert))?;

    Ok(http_resp(
        ResetPasswordRequestResponse {
            user_email: receipt.user_email,
            verify_by: receipt.verify_by,
        },
        (),
        start,
    ))
}
