//! Authenticated self-service soft deletion.

use std::sync::Arc;

use axum::{Extension, Json, extract::State, response::IntoResponse};
use uuid::Uuid;

use crate::{
    dto::{
        requests::auth::delete_account_request::DeleteAccountRequest,
        responses::{
            auth::delete_account_response::DeleteAccountResponse,
            response_data::http_resp_with_cookies,
        },
    },
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    features::accounts::api::{
        account_error::{AccountMutation, map_account_error},
        logout::removal_cookie,
    },
    features::accounts::error::AccountError,
    init::state::ServerState,
    util::time::now::tokio_now,
};

#[utoipa::path(
    delete,
    path = "/api/auth/account",
    tag = "auth",
    request_body = DeleteAccountRequest,
    responses(
        (status = 200, description = "Account anonymized and retention scheduled", body = DeleteAccountResponse),
        (status = 400, description = "Invalid lifecycle request or password confirmation", body = CodeErrorResp),
        (status = 401, description = "Authentication required", body = CodeErrorResp),
        (status = 403, description = "Protected system actor", body = CodeErrorResp),
        (status = 409, description = "Account lifecycle conflict", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn delete_account(
    Extension(user_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    Json(request): Json<DeleteAccountRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let receipt = state
        .account_service()
        .soft_delete_account(user_id, &request.current_password)
        .await
        .map_err(map_delete_account_error)?;

    Ok(http_resp_with_cookies(
        DeleteAccountResponse {
            user_id: receipt.user_id,
            deleted_at: receipt.deleted_at,
            purge_after: receipt.purge_after,
        },
        (),
        start,
        None,
        Some(vec![removal_cookie()]),
    ))
}

fn map_delete_account_error(error: AccountError) -> CodeErrorResp {
    match error {
        AccountError::WrongPassword => code_err(
            CodeError::ACCOUNT_PASSWORD_CONFIRMATION_FAILED,
            AccountError::WrongPassword,
        ),
        error => map_account_error(error, AccountMutation::Update),
    }
}
