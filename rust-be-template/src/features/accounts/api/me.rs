//! Current-account transport endpoint.

use std::sync::Arc;

use axum::{Extension, extract::State, response::IntoResponse};

use crate::{
    build_info::{BUILD_TIME_UTC, LIB_VERSION_MAP, RUSTC_VERSION},
    dto::responses::{
        auth::me_response::{MeResponse, UserInfo, UserProfilePicture},
        response_data::http_resp,
    },
    errors::code_error::{CodeErrorResp, HandlerResponse},
    features::accounts::api::account_error::{AccountMutation, map_account_error},
    init::state::ServerState,
    routers::middleware::is_logged_in::AuthStatus,
    util::time::now::tokio_now,
};

#[utoipa::path(
    get,
    path = "/api/auth/me",
    tag = "auth",
    responses(
        (status = 200, description = "Current user information", body = MeResponse),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn me_handler(
    Extension(auth_status): Extension<AuthStatus>,
    State(state): State<Arc<ServerState>>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let account = match auth_status {
        AuthStatus::LoggedIn(user_id) => state
            .account_service()
            .current_account(user_id)
            .await
            .map_err(|error| map_account_error(error, AccountMutation::Update))?,
        AuthStatus::LoggedOut => None,
    };
    let (user_info, user_profile_picture) = match account {
        Some(account) => (
            Some(UserInfo::from(account.profile)),
            account.profile_picture.map(UserProfilePicture::from),
        ),
        None => (None, None),
    };

    Ok(http_resp(
        MeResponse {
            user_info,
            user_profile_picture,
            build_time: BUILD_TIME_UTC,
            axum_version: axum_version(),
            rust_version: RUSTC_VERSION,
        },
        (),
        start,
    ))
}

fn axum_version() -> String {
    match LIB_VERSION_MAP.get("axum") {
        Some(version) => [version.get_name(), version.get_version()].concat(),
        None => "Unknown".to_string(),
    }
}
