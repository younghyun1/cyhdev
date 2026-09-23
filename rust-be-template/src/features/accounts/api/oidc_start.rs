//! Same-origin entry points for login and account-link authorization flows.
//!
//! Each start sets a browser-binding cookie. It must be `SameSite=Lax`, not `Strict`: the
//! provider's redirect back to the callback is a cross-site top-level navigation, and only
//! Lax cookies accompany it. The pending flow keeps just the cookie's SHA-256.

use std::{net::SocketAddr, sync::Arc};

use axum::{
    Extension, Json,
    extract::{ConnectInfo, State},
    http::HeaderMap,
    response::IntoResponse,
};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use uuid::Uuid;
use zeroize::Zeroize;

use crate::{
    dto::{
        requests::auth::oidc_request::OidcLinkStartRequest,
        responses::{
            auth::oidc_response::OidcAuthorizationResponse,
            response_data::http_resp_with_cookies_sensitive,
        },
    },
    errors::code_error::{CodeErrorResp, HandlerResponse},
    features::accounts::{
        api::{
            account_error::{AccountMutation, map_account_error},
            auth_abuse::request_client_ip,
            login::session_token_from_cookie,
            password_confirmation::Confirmation,
        },
        domain::oidc::{OIDC_BINDING_COOKIE_NAME, OIDC_BINDING_COOKIE_SECONDS, OidcFlowMode},
        error::AccountError,
    },
    init::state::ServerState,
    util::time::now::tokio_now,
};

#[utoipa::path(
    post,
    path = "/api/auth/oidc/login/start",
    tag = "auth",
    responses(
        (status = 200, description = "OIDC login authorization URL; sets the browser-binding cookie", body = OidcAuthorizationResponse),
        (status = 404, description = "OIDC is not configured"),
        (status = 429, description = "Authentication attempt budget exhausted"),
        (status = 503, description = "OIDC temporarily unavailable")
    )
)]
pub async fn start_oidc_login(
    State(state): State<Arc<ServerState>>,
) -> HandlerResponse<impl IntoResponse> {
    start_authorization(state, OidcFlowMode::Login).await
}

#[utoipa::path(
    post,
    path = "/api/auth/oidc/link/start",
    tag = "auth",
    request_body = OidcLinkStartRequest,
    responses(
        (status = 200, description = "OIDC account-link authorization URL; sets the browser-binding cookie", body = OidcAuthorizationResponse),
        (status = 400, description = "Current password rejected", body = CodeErrorResp),
        (status = 401, description = "Verified local session required, or revoked after repeated wrong passwords", body = CodeErrorResp),
        (status = 404, description = "OIDC is not configured", body = CodeErrorResp),
        (status = 409, description = "No local password is available to confirm", body = CodeErrorResp),
        (status = 429, description = "Authentication or password confirmation budget exhausted", body = CodeErrorResp),
        (status = 503, description = "OIDC temporarily unavailable", body = CodeErrorResp)
    )
)]
pub async fn start_oidc_link(
    Extension(user_id): Extension<Uuid>,
    ConnectInfo(socket_addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    cookie_jar: CookieJar,
    State(state): State<Arc<ServerState>>,
    Json(mut request): Json<OidcLinkStartRequest>,
) -> HandlerResponse<impl IntoResponse> {
    if !state.oidc_service().is_enabled() {
        request.zeroize();
        return Err(map_start_error(AccountError::OidcDisabled));
    }
    let confirmation = Confirmation {
        user_id,
        client_ip: request_client_ip(&headers, socket_addr),
    };
    confirmation.admit(&state).await?;
    let result = state
        .account_service()
        .confirm_oidc_link_start(user_id, &request.current_password)
        .await;
    request.zeroize();
    confirmation
        .settle(
            &state,
            session_token_from_cookie(&cookie_jar),
            result,
            map_start_error,
        )
        .await?;
    start_authorization(
        state,
        OidcFlowMode::Link {
            expected_user_id: user_id,
        },
    )
    .await
}

async fn start_authorization(
    state: Arc<ServerState>,
    mode: OidcFlowMode,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let started = state
        .oidc_service()
        .start_authorization(mode)
        .await
        .map_err(map_start_error)?;
    Ok(http_resp_with_cookies_sensitive(
        OidcAuthorizationResponse {
            authorization_url: started.authorization_url,
        },
        (),
        start,
        Some(vec![binding_cookie(started.browser_binding.expose())]),
        None,
    ))
}

fn map_start_error(error: AccountError) -> CodeErrorResp {
    map_account_error(error, AccountMutation::Update)
}

/// Host-only, script-inaccessible binding for one pending authorization.
pub(super) fn binding_cookie(value: &str) -> Cookie<'static> {
    Cookie::build((OIDC_BINDING_COOKIE_NAME, value.to_owned()))
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::seconds(OIDC_BINDING_COOKIE_SECONDS))
        .build()
}

/// Clears the binding cookie; the callback sends it on every outcome.
pub(super) fn binding_removal_cookie() -> Cookie<'static> {
    let mut cookie = binding_cookie("");
    cookie.make_removal();
    cookie
}

#[cfg(test)]
mod tests {
    use super::{binding_cookie, binding_removal_cookie};
    use crate::features::accounts::domain::oidc::OIDC_BINDING_COOKIE_NAME;
    use axum_extra::extract::cookie::SameSite;

    #[test]
    fn binding_cookie_is_host_only_lax_and_short_lived() {
        let cookie = binding_cookie("binding");
        assert!(OIDC_BINDING_COOKIE_NAME.starts_with("__Host-"));
        assert_eq!(cookie.name(), OIDC_BINDING_COOKIE_NAME);
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.domain(), None);
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.secure(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
        assert_eq!(
            cookie.max_age().map(|duration| duration.whole_seconds()),
            Some(600)
        );
        let removal = binding_removal_cookie();
        assert_eq!(removal.path(), Some("/"));
        assert_eq!(
            removal.max_age().map(|duration| duration.whole_seconds()),
            Some(0)
        );
    }
}
