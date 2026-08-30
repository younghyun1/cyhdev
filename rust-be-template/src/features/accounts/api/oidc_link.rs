//! Same-origin completion and password-confirmed unlink operations.

use std::sync::Arc;

use axum::{Extension, Json, extract::State, response::IntoResponse};
use axum_extra::extract::CookieJar;
use uuid::Uuid;
use zeroize::Zeroize;

use crate::{
    dto::{
        requests::auth::oidc_request::{OidcLinkCompleteRequest, OidcUnlinkRequest},
        responses::{
            auth::oidc_response::OidcLinkResponse,
            response_data::http_resp_with_cookies_sensitive,
        },
    },
    errors::code_error::HandlerResponse,
    features::accounts::api::{
        account_error::{AccountMutation, map_account_error},
        login::{session_cookie, session_token_from_cookie},
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
        return Err(map_account_error(
            crate::features::accounts::error::AccountError::OidcFlowRejected,
            AccountMutation::Update,
        ));
    }
    let completion = state
        .oidc_service()
        .consume_link_completion(&request.completion_token)
        .await;
    request.zeroize();
    let (expected_user_id, identity) =
        completion.map_err(|error| map_account_error(error, AccountMutation::Update))?;
    let receipt = state
        .account_service()
        .complete_oidc_link(
            user_id,
            expected_user_id,
            &identity,
            session_token_from_cookie(&cookie_jar),
        )
        .await
        .map_err(|error| map_account_error(error, AccountMutation::Update))?;
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
        (status = 401, description = "Verified local session required"),
        (status = 409, description = "Another usable login method is required")
    )
)]
pub async fn unlink_oidc(
    Extension(user_id): Extension<Uuid>,
    cookie_jar: CookieJar,
    State(state): State<Arc<ServerState>>,
    Json(mut request): Json<OidcUnlinkRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let oidc = state.oidc_service();
    let issuer = oidc
        .issuer()
        .ok_or_else(|| map_account_error(
            crate::features::accounts::error::AccountError::OidcDisabled,
            AccountMutation::Update,
        ))?;
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
    let receipt = result.map_err(|error| map_account_error(error, AccountMutation::Update))?;
    Ok(http_resp_with_cookies_sensitive(
        OidcLinkResponse { linked: false },
        (),
        start,
        Some(vec![session_cookie(&receipt.session_token)]),
        None,
    ))
}
