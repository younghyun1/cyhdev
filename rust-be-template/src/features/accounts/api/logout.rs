use std::sync::Arc;

use axum::{extract::State, response::IntoResponse};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use tracing::{info, warn};

use crate::{
    dto::responses::{
        auth::logout_response::LogoutResponse, response_data::http_resp_with_cookies,
    },
    errors::code_error::{CodeErrorResp, HandlerResponse},
    features::accounts::domain::session::SESSION_COOKIE_NAME,
    init::state::ServerState,
    util::time::now::tokio_now,
};

#[utoipa::path(
    post,
    path = "/api/auth/logout",
    tag = "auth",
    responses(
        (status = 200, description = "Logout successful", body = LogoutResponse),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn logout(
    cookie_jar: CookieJar,
    State(state): State<Arc<ServerState>>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let cookie = removal_cookie();

    if let Some(session_cookie) = cookie_jar.get(SESSION_COOKIE_NAME) {
        if state
            .account_service()
            .logout(session_cookie.value())
            .await
        {
            info!("User logout; session removed");
        } else {
            warn!("Logout session was absent or malformed");
        }
    }

    Ok(http_resp_with_cookies(
        LogoutResponse {
            message: "Logout successful".to_string(),
        },
        (),
        start,
        None,
        Some(vec![cookie]),
    ))
}

fn removal_cookie() -> Cookie<'static> {
    let mut cookie = Cookie::build((SESSION_COOKIE_NAME, ""))
        .path("/")
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .build();
    cookie.make_removal();
    cookie
}

#[cfg(test)]
mod tests {
    use super::removal_cookie;
    use crate::features::accounts::domain::session::SESSION_COOKIE_NAME;

    #[test]
    fn removal_cookie_matches_session_cookie_scope() {
        let cookie = removal_cookie();

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
            Some(0)
        );
    }
}
