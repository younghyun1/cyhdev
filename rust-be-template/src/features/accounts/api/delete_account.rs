//! Authenticated self-service soft deletion.

use std::{net::SocketAddr, sync::Arc};

use axum::{
    Extension, Json,
    extract::{ConnectInfo, State},
    http::HeaderMap,
    response::IntoResponse,
};
use axum_extra::extract::CookieJar;
use uuid::Uuid;

use crate::{
    dto::{
        requests::auth::delete_account_request::DeleteAccountRequest,
        responses::{
            auth::delete_account_response::DeleteAccountResponse,
            response_data::http_resp_with_cookies,
        },
    },
    errors::code_error::{CodeErrorResp, HandlerResponse},
    features::accounts::api::{
        account_error::{AccountMutation, map_account_error},
        auth_abuse::request_client_ip,
        login::session_token_from_cookie,
        logout::removal_cookie,
        password_confirmation::Confirmation,
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
        (status = 429, description = "Password confirmation budget exhausted", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn delete_account(
    Extension(user_id): Extension<Uuid>,
    ConnectInfo(socket_addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    cookie_jar: CookieJar,
    State(state): State<Arc<ServerState>>,
    Json(request): Json<DeleteAccountRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let confirmation = Confirmation {
        user_id,
        client_ip: request_client_ip(&headers, socket_addr),
    };
    confirmation.admit(&state).await?;
    let result = state
        .account_service()
        .soft_delete_account(user_id, &request.current_password)
        .await;
    let receipt = confirmation
        .settle(
            &state,
            session_token_from_cookie(&cookie_jar),
            result,
            map_delete_account_error,
        )
        .await?;

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
    map_account_error(error, AccountMutation::Update)
}
