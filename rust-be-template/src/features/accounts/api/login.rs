use std::sync::Arc;

use axum::{Json, extract::State, response::IntoResponse};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use zeroize::Zeroize;

use crate::{
    dto::{
        requests::auth::login_request::LoginRequest,
        responses::{auth::login_response::LoginResponse, response_data::http_resp_with_cookies},
    },
    errors::code_error::HandlerResponse,
    features::accounts::{
        api::account_error::{AccountMutation, map_account_error},
        domain::session::{SESSION_COOKIE_NAME, SessionToken},
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
        (status = 404, description = "User not found"),
        (status = 503, description = "Session capacity unavailable"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn login(
    cookie_jar: CookieJar,
    State(state): State<Arc<ServerState>>,
    Json(mut request): Json<LoginRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let previous_session_token = session_token_from_cookie(&cookie_jar);
    let login_result = state
        .account_service()
        .login(
            &request.user_email,
            &request.user_password,
            previous_session_token,
        )
        .await;
    request.zeroize();
    let receipt =
        login_result.map_err(|error| map_account_error(error, AccountMutation::Update))?;

    let cookie = session_cookie(&receipt.session_token);
    Ok(http_resp_with_cookies(
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

fn session_token_from_cookie(cookie_jar: &CookieJar) -> Option<&str> {
    cookie_jar
        .get(SESSION_COOKIE_NAME)
        .map(|cookie| cookie.value())
}

fn session_cookie(session_token: &SessionToken) -> Cookie<'static> {
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
