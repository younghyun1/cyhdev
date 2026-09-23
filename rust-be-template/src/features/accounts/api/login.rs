use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use axum::{
    Json,
    extract::{ConnectInfo, State},
    http::HeaderMap,
    response::IntoResponse,
};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use zeroize::Zeroize;

use crate::{
    dto::{
        requests::auth::login_request::LoginRequest,
        responses::{
            auth::login_response::LoginResponse, response_data::http_resp_with_cookies_sensitive,
        },
    },
    errors::code_error::HandlerResponse,
    features::accounts::{
        api::{
            account_error::map_login_error,
            auth_abuse::{map_auth_throttle_rejection, request_client_ip},
        },
        domain::{
            auth_abuse::{AuthEndpoint, AuthIdentity},
            session::{SESSION_COOKIE_NAME, SessionToken},
        },
        error::AccountError,
        service::auth_abuse::AuthAbuseService,
    },
    init::state::ServerState,
    util::time::now::tokio_now,
};

#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Correct password, but the email address is not verified; no session is issued"),
        (status = 429, description = "Authentication attempt budget exhausted"),
        (status = 503, description = "Session capacity unavailable"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn login(
    cookie_jar: CookieJar,
    ConnectInfo(socket_addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    State(state): State<Arc<ServerState>>,
    Json(mut request): Json<LoginRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let previous_session_token = session_token_from_cookie(&cookie_jar);
    let client_ip = request_client_ip(&headers, socket_addr);
    let abuse = state.auth_abuse_service();
    // Identity budgets count failures only, so a correct password is never locked out by
    // someone who merely knows the email; the source-IP layer still counts every attempt.
    for identity in login_identities(&request.user_email, client_ip) {
        abuse
            .ensure_failure_budget(AuthEndpoint::Login, identity)
            .await
            .map_err(map_auth_throttle_rejection)?;
    }
    let login_result = state
        .account_service()
        .login(
            &request.user_email,
            &request.user_password,
            previous_session_token,
        )
        .await;
    settle_login_budgets(&abuse, &request.user_email, client_ip, &login_result).await;
    request.zeroize();
    let receipt = login_result.map_err(map_login_error)?;

    let cookie = session_cookie(&receipt.session_token);
    Ok(http_resp_with_cookies_sensitive(
        LoginResponse {
            message: "Login successful".to_string(),
            user_id: receipt.user_id,
        },
        (),
        start,
        Some(vec![cookie]),
        None,
    ))
}

/// The strict per-source and the looser account-wide failure budgets for one login.
fn login_identities(email: &str, client_ip: IpAddr) -> [AuthIdentity<'_>; 2] {
    [
        AuthIdentity::EmailFromIp(email, client_ip),
        AuthIdentity::Email(email),
    ]
}

/// Charges a rejected password to both budgets, or clears the per-source budget on success.
async fn settle_login_budgets<T>(
    abuse: &AuthAbuseService,
    email: &str,
    client_ip: IpAddr,
    result: &Result<T, AccountError>,
) {
    match result {
        Err(AccountError::InvalidCredentials) => {
            for identity in login_identities(email, client_ip) {
                if let Err(rejection) = abuse.record_failure(AuthEndpoint::Login, identity).await {
                    tracing::warn!(
                        event = "auth_failure_not_recorded",
                        endpoint = rejection.endpoint().as_str(),
                        dimension = rejection.dimension().as_str(),
                        capacity_saturated = rejection.capacity_saturated(),
                        "Login failure could not be recorded"
                    );
                }
            }
        }
        Ok(_) => {
            abuse
                .forget_identity(
                    AuthEndpoint::Login,
                    AuthIdentity::EmailFromIp(email, client_ip),
                )
                .await;
        }
        Err(_) => {}
    }
}

pub(super) fn session_token_from_cookie(cookie_jar: &CookieJar) -> Option<&str> {
    cookie_jar
        .get(SESSION_COOKIE_NAME)
        .map(|cookie| cookie.value())
}

pub(super) fn session_cookie(session_token: &SessionToken) -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE_NAME, session_token.expose().to_owned()))
        .path("/")
        .http_only(true)
        .max_age(time::Duration::hours(1))
        .same_site(axum_extra::extract::cookie::SameSite::Strict)
        .secure(true)
        .build()
}

#[cfg(test)]
mod tests {
    use super::session_cookie;
    use crate::features::accounts::domain::session::{
        SESSION_COOKIE_NAME, SESSION_SECRET_BYTES, SessionToken,
    };

    #[test]
    fn session_cookie_enforces_host_prefix_attributes() {
        let token = SessionToken::from_secret(&[7; SESSION_SECRET_BYTES]);
        let cookie = session_cookie(&token);

        assert_eq!(cookie.name(), SESSION_COOKIE_NAME);
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.domain(), None);
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.secure(), Some(true));
        assert_eq!(
            cookie.same_site(),
            Some(axum_extra::extract::cookie::SameSite::Strict)
        );
        assert_eq!(
            cookie.max_age().map(|duration| duration.whole_seconds()),
            Some(3_600)
        );
    }
}
