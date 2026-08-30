use std::sync::Arc;

use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse},
};

use crate::{
    dto::{
        requests::auth::verify_user_email_request::EmailValidationToken,
        responses::auth::email_validate_response::{
            EmailValidateResponse, hydrate_email_validate_response_page,
        },
    },
    errors::code_error::{CodeErrorResp, HandlerResponse},
    features::accounts::api::account_error::{AccountMutation, map_account_error},
    init::state::ServerState,
    util::time::now::tokio_now,
};

#[utoipa::path(
    get,
    path = "/api/auth/verify-user-email",
    tag = "auth",
    params(
        ("email_validation_token_id" = uuid::Uuid, Query, description = "Email validation token ID")
    ),
    responses(
        (status = 200, description = "Email verified successfully", body = String, content_type = "text/html"),
        (status = 400, description = "Invalid token or already verified", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn verify_user_email(
    State(state): State<Arc<ServerState>>,
    Query(token): Query<EmailValidationToken>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let receipt = state
        .account_service()
        .verify_email(token.email_validation_token_id)
        .await
        .map_err(|error| map_account_error(error, AccountMutation::Update))?;
    let response = EmailValidateResponse {
        user_email: receipt.user_email,
        verified_at: receipt.verified_at,
        time_to_process: start.elapsed(),
    };

    Ok(Html(hydrate_email_validate_response_page(&response)))
}
