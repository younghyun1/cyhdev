//! Public provider availability and current-account link status.

use std::sync::Arc;

use axum::{Extension, extract::State, response::IntoResponse};

use crate::{
    dto::responses::{
        auth::oidc_response::OidcStatusResponse,
        response_data::http_resp_sensitive,
    },
    errors::code_error::HandlerResponse,
    features::accounts::api::account_error::{AccountMutation, map_account_error},
    init::state::ServerState,
    routers::middleware::is_logged_in::AuthStatus,
    util::time::now::tokio_now,
};

#[utoipa::path(
    get,
    path = "/api/auth/oidc/status",
    tag = "auth",
    responses((status = 200, description = "OIDC provider and link status", body = OidcStatusResponse))
)]
pub async fn oidc_status(
    Extension(auth_status): Extension<AuthStatus>,
    State(state): State<Arc<ServerState>>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let oidc = state.oidc_service();
    let issuer = oidc.issuer();
    let linked = match (auth_status, issuer) {
        (AuthStatus::LoggedIn(user_id), Some(issuer)) => state
            .account_service()
            .oidc_is_linked(user_id, issuer)
            .await
            .map_err(|error| map_account_error(error, AccountMutation::Update))?,
        (AuthStatus::LoggedIn(_) | AuthStatus::LoggedOut, None)
        | (AuthStatus::LoggedOut, Some(_)) => false,
    };
    Ok(http_resp_sensitive(
        OidcStatusResponse {
            enabled: oidc.is_enabled(),
            provider_name: oidc.provider_name().map(str::to_owned),
            linked,
        },
        (),
        start,
    ))
}
