use std::sync::Arc;

use axum::{
    Json,
    extract::{State, rejection::JsonRejection},
    response::{IntoResponse, Response},
};

use crate::{
    dto::{
        requests::auth::verify_user_email_request::VerifyUserEmailRequest,
        responses::{
            auth::verify_user_email_response::VerifyUserEmailResponse,
            response_data::http_resp_sensitive,
        },
    },
    errors::code_error::{CodeError, CodeErrorResp, code_err},
    features::accounts::{
        api::{
            account_error::{AccountMutation, map_account_error},
            auth_abuse::map_auth_throttle_rejection,
        },
        domain::auth_abuse::{AuthEndpoint, AuthIdentity},
        error::AccountError,
    },
    init::state::ServerState,
    util::time::now::tokio_now,
};

/// Consumes an admitted one-time token after explicit browser confirmation.
#[utoipa::path(
    post,
    path = "/api/auth/verify-user-email",
    tag = "auth",
    request_body = VerifyUserEmailRequest,
    responses(
        (status = 200, description = "Email verified after explicit confirmation", body = VerifyUserEmailResponse),
        (status = 400, description = "Invalid or unavailable verification token", body = CodeErrorResp),
        (status = 429, description = "Verification attempt budget exhausted", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn verify_user_email(
    State(state): State<Arc<ServerState>>,
    request: Result<Json<VerifyUserEmailRequest>, JsonRejection>,
) -> Response {
    let start = tokio_now();
    let Json(request) = match request {
        Ok(request) => request,
        Err(_rejection) => {
            return code_err(
                CodeError::INVALID_EMAIL_VERIFICATION_TOKEN,
                "malformed email verification request",
            )
            .into_response();
        }
    };
    let token = request.email_validation_token_id;
    match state
        .auth_abuse_service()
        .check_identity(
            AuthEndpoint::EmailVerification,
            AuthIdentity::Token(token.as_bytes()),
        )
        .await
    {
        Ok(()) => {}
        Err(rejection) => {
            return map_auth_throttle_rejection(rejection).into_response();
        }
    }
    let result = state.account_service().verify_email(token).await;
    match result {
        Ok(receipt) => http_resp_sensitive(
            VerifyUserEmailResponse {
                verified_at: receipt.verified_at,
            },
            (),
            start,
        )
        .into_response(),
        Err(error) => map_verification_error(error).into_response(),
    }
}

fn map_verification_error(error: AccountError) -> CodeErrorResp {
    match error {
        AccountError::EmailVerificationTokenNotFound
        | AccountError::EmailVerificationTokenExpired
        | AccountError::EmailVerificationTokenFabricated
        | AccountError::EmailVerificationTokenAlreadyUsed
        | AccountError::TokenAlreadyConsumed
        | AccountError::EmailAlreadyVerified => {
            code_err(CodeError::INVALID_EMAIL_VERIFICATION_TOKEN, error)
        }
        error => map_account_error(error, AccountMutation::Update),
    }
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;

    use super::map_verification_error;
    use crate::{
        errors::code_error::CodeError,
        features::accounts::error::AccountError,
    };

    #[test]
    fn public_token_failures_share_one_response_contract() {
        for error in [
            AccountError::EmailVerificationTokenNotFound,
            AccountError::EmailVerificationTokenExpired,
            AccountError::EmailVerificationTokenFabricated,
            AccountError::EmailVerificationTokenAlreadyUsed,
            AccountError::TokenAlreadyConsumed,
            AccountError::EmailAlreadyVerified,
        ] {
            let response = map_verification_error(error);
            assert_eq!(response.error_code, CodeError::INVALID_EMAIL_VERIFICATION_TOKEN.error_code);
            assert_eq!(response.http_status_code, StatusCode::BAD_REQUEST);
            assert_eq!(
                response.message,
                CodeError::INVALID_EMAIL_VERIFICATION_TOKEN.message
            );
        }
    }
}
