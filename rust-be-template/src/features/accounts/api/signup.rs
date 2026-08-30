use std::{mem, sync::Arc};

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use zeroize::Zeroizing;

use crate::{
    dto::{
        requests::auth::signup_request::SignupRequest,
        responses::{
            auth::signup_response::SignupResponse,
            response_data::http_resp_sensitive,
        },
    },
    errors::code_error::{CodeErrorResp, HandlerResponse},
    features::accounts::{
        api::{
            account_error::map_signup_error,
            auth_abuse::map_auth_throttle_rejection,
        },
        domain::{
            account::SignupCommand,
            auth_abuse::{AuthEndpoint, AuthIdentity},
        },
        error::AccountError,
    },
    init::state::ServerState,
    util::time::now::tokio_now,
};

const SIGNUP_RESPONSE_FLOOR: std::time::Duration = std::time::Duration::from_millis(300);

#[utoipa::path(
    post,
    path = "/api/auth/signup",
    tag = "auth",
    request_body = SignupRequest,
    responses(
        (status = 202, description = "Registration request accepted", body = SignupResponse),
        (status = 400, description = "Invalid registration input", body = CodeErrorResp),
        (status = 409, description = "Public account identity unavailable", body = CodeErrorResp),
        (status = 429, description = "Registration attempt budget exhausted", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn signup_handler(
    State(state): State<Arc<ServerState>>,
    Json(mut request): Json<SignupRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let abuse = state.auth_abuse_service();
    abuse
        .check_identity(
            AuthEndpoint::Signup,
            AuthIdentity::Email(&request.user_email),
        )
        .await
        .map_err(map_auth_throttle_rejection)?;
    abuse
        .check_identity(
            AuthEndpoint::Signup,
            AuthIdentity::UserName(&request.user_name),
        )
        .await
        .map_err(map_auth_throttle_rejection)?;
    let start = tokio_now();
    let duplicate_email = Zeroizing::new(request.user_email.clone());
    let command = SignupCommand {
        user_name: mem::take(&mut request.user_name),
        user_email: mem::take(&mut request.user_email),
        password: Zeroizing::new(mem::take(&mut request.user_password)),
        country: request.user_country,
        language: request.user_language,
        subdivision: request.user_subdivision,
    };
    let result = state.account_service().signup(command).await;
    match result {
        Ok(_) => {}
        Err(AccountError::DuplicateEmail(_)) => {
            if let Err(error) = state
                .account_service()
                .resend_verification_for_duplicate_email(&duplicate_email)
                .await
            {
                tracing::error!(
                    event = "signup_verification_reissue_failed",
                    error = %error,
                    "Failed to reissue signup verification capability"
                );
            }
        }
        Err(error @ AccountError::DuplicateUserName(_)) => {
            tokio::time::sleep_until(start + SIGNUP_RESPONSE_FLOOR).await;
            return Err(map_signup_error(error));
        }
        Err(error) => return Err(map_signup_error(error)),
    }
    tokio::time::sleep_until(start + SIGNUP_RESPONSE_FLOOR).await;

    Ok((StatusCode::ACCEPTED, http_resp_sensitive(
        SignupResponse {
            message: "If registration can proceed, verification instructions will be sent."
                .to_owned(),
        },
        (),
        start,
    )))
}
