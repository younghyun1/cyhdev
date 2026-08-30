use std::sync::Arc;

use axum::{Json, extract::State, response::IntoResponse};

use crate::{
    dto::{
        requests::auth::check_if_user_exists_request::CheckIfUserExistsRequest,
        responses::response_data::http_resp,
    },
    errors::code_error::{CodeErrorResp, HandlerResponse},
    features::accounts::api::account_error::{AccountMutation, map_account_error},
    init::state::ServerState,
    util::time::now::tokio_now,
};

#[derive(serde_derive::Serialize, utoipa::ToSchema)]
pub struct CheckIfUserExistsResponse {
    pub email_exists: bool,
}

#[utoipa::path(
    post,
    path = "/api/auth/check-if-user-exists",
    tag = "auth",
    request_body = CheckIfUserExistsRequest,
    responses(
        (status = 200, description = "Check if user exists", body = CheckIfUserExistsResponse),
        (status = 400, description = "Invalid input", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn check_if_user_exists_handler(
    State(state): State<Arc<ServerState>>,
    Json(request): Json<CheckIfUserExistsRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();

    let email_exists = state
        .account_service()
        .email_exists(&request.user_email)
        .await
        .map_err(|error| map_account_error(error, AccountMutation::Update))?;

    Ok(http_resp(
        CheckIfUserExistsResponse { email_exists },
        (),
        start,
    ))
}
