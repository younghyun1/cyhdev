use std::{mem, sync::Arc};

use axum::{Extension, Json, extract::State, response::IntoResponse};
use chrono::{DateTime, Utc};
use zeroize::Zeroizing;

use crate::{
    dto::{
        requests::auth::signup_request::SignupRequest,
        responses::{auth::signup_response::SignupResponse, response_data::http_resp},
    },
    errors::code_error::{CodeErrorResp, HandlerResponse},
    features::accounts::{
        api::account_error::{AccountMutation, map_account_error},
        domain::account::SignupCommand,
    },
    init::state::ServerState,
    util::time::now::tokio_now,
};

#[utoipa::path(
    post,
    path = "/api/auth/signup",
    tag = "auth",
    request_body = SignupRequest,
    responses(
        (status = 200, description = "User successfully signed up", body = SignupResponse),
        (status = 400, description = "Invalid input or account identity already exists", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn signup_handler(
    Extension(_request_received_time): Extension<DateTime<Utc>>,
    State(state): State<Arc<ServerState>>,
    Json(mut request): Json<SignupRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let command = SignupCommand {
        user_name: mem::take(&mut request.user_name),
        user_email: mem::take(&mut request.user_email),
        password: Zeroizing::new(mem::take(&mut request.user_password)),
        country: request.user_country,
        language: request.user_language,
        subdivision: request.user_subdivision,
    };
    let receipt = state
        .account_service()
        .signup(command)
        .await
        .map_err(|error| map_account_error(error, AccountMutation::Insert))?;

    Ok(http_resp(
        SignupResponse {
            user_name: receipt.user_name,
            user_email: receipt.user_email,
            verify_by: receipt.verify_by,
        },
        (),
        start,
    ))
}
