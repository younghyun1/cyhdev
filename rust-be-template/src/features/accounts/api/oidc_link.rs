//! Same-origin completion and password-confirmed unlink operations.

use std::{net::SocketAddr, sync::Arc};

use axum::{
    Extension, Json,
    extract::{ConnectInfo, State},
    http::HeaderMap,
    response::IntoResponse,
};
use axum_extra::extract::CookieJar;
use uuid::Uuid;
use zeroize::Zeroize;

use crate::{
    dto::{
        requests::auth::oidc_request::{OidcLinkCompleteRequest, OidcUnlinkRequest},
        responses::{
            auth::oidc_response::OidcLinkResponse, response_data::http_resp_with_cookies_sensitive,
        },
    },
    errors::code_error::{CodeErrorResp, HandlerResponse},
    features::accounts::{
        api::{
            account_error::{AccountMutation, map_account_error},
            auth_abuse::request_client_ip,
            login::{session_cookie, session_token_from_cookie},
            password_confirmation::Confirmation,
        },
        error::AccountError,
    },
    init::state::ServerState,
    util::time::now::tokio_now,
};

const COMPLETION_TOKEN_LENGTH: usize = 43;

#[utoipa::path(
    post,
    path = "/api/auth/oidc/link/complete",
    tag = "auth",
    request_body = OidcLinkCompleteRequest,
    responses(
        (status = 200, description = "OIDC identity linked", body = OidcLinkResponse),
        (status = 400, description = "Completion token invalid or consumed"),
        (status = 401, description = "Verified local session required"),
        (status = 409, description = "Identity conflicts with an existing link")
    )
)]
pub async fn complete_oidc_link(
    Extension(user_id): Extension<Uuid>,
    cookie_jar: CookieJar,
    State(state): State<Arc<ServerState>>,
    Json(mut request): Json<OidcLinkCompleteRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    if request.completion_token.len() != COMPLETION_TOKEN_LENGTH {
        request.zeroize();
        return Err(map_link_error(AccountError::OidcFlowRejected));
    }
    let completion = state
        .oidc_service()
        .consume_link_completion(&request.completion_token)
        .await;
    request.zeroize();
    let (expected_user_id, identity) = completion.map_err(map_link_error)?;
    let oidc = state.oidc_service();
    let provider_name = oidc.provider_name().unwrap_or("OpenID Connect");
    let receipt = state
        .account_service()
        .complete_oidc_link(
            user_id,
            expected_user_id,
            &identity,
            provider_name,
            session_token_from_cookie(&cookie_jar),
        )
        .await
        .map_err(map_link_error)?;
    Ok(http_resp_with_cookies_sensitive(
        OidcLinkResponse { linked: true },
        (),
        start,
        Some(vec![session_cookie(&receipt.session_token)]),
        None,
    ))
}

#[utoipa::path(
    delete,
    path = "/api/auth/oidc/link",
    tag = "auth",
    request_body = OidcUnlinkRequest,
    responses(
        (status = 200, description = "OIDC identity unlinked", body = OidcLinkResponse),
        (status = 400, description = "Current password rejected"),
        (status = 401, description = "Verified local session required, or revoked after repeated wrong passwords"),
        (status = 409, description = "Another usable login method is required"),
        (status = 429, description = "Password confirmation budget exhausted")
    )
)]
pub async fn unlink_oidc(
    Extension(user_id): Extension<Uuid>,
    ConnectInfo(socket_addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    cookie_jar: CookieJar,
    State(state): State<Arc<ServerState>>,
    Json(mut request): Json<OidcUnlinkRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let oidc = state.oidc_service();
    let issuer = oidc
        .issuer()
        .ok_or_else(|| map_link_error(AccountError::OidcDisabled))?;
    let confirmation = Confirmation {
        user_id,
        client_ip: request_client_ip(&headers, socket_addr),
    };
    confirmation.admit(&state).await?;
    let result = state
        .account_service()
        .unlink_oidc(
            user_id,
            issuer,
            &request.current_password,
            session_token_from_cookie(&cookie_jar),
        )
        .await;
    request.zeroize();
    let receipt = confirmation
        .settle(
            &state,
            session_token_from_cookie(&cookie_jar),
            result,
            map_link_error,
        )
        .await?;
    Ok(http_resp_with_cookies_sensitive(
        OidcLinkResponse { linked: false },
        (),
        start,
        Some(vec![session_cookie(&receipt.session_token)]),
        None,
    ))
}

fn map_link_error(error: AccountError) -> CodeErrorResp {
    map_account_error(error, AccountMutation::Update)
}
