//! Same-origin entry points for login and account-link authorization flows.

use std::sync::Arc;

use axum::{Extension, extract::State, response::IntoResponse};
use uuid::Uuid;

use crate::{
    dto::responses::{
        auth::oidc_response::OidcAuthorizationResponse, response_data::http_resp_sensitive,
    },
    errors::code_error::HandlerResponse,
    features::accounts::{
        api::account_error::{AccountMutation, map_account_error},
        domain::oidc::OidcFlowMode,
    },
    init::state::ServerState,
    util::time::now::tokio_now,
};

#[utoipa::path(
    post,
    path = "/api/auth/oidc/login/start",
    tag = "auth",
    responses(
        (status = 200, description = "OIDC login authorization URL", body = OidcAuthorizationResponse),
        (status = 404, description = "OIDC is not configured"),
        (status = 429, description = "Authentication attempt budget exhausted"),
        (status = 503, description = "OIDC flow capacity unavailable")
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
    responses(
        (status = 200, description = "OIDC account-link authorization URL", body = OidcAuthorizationResponse),
        (status = 401, description = "Verified local session required"),
        (status = 404, description = "OIDC is not configured"),
        (status = 429, description = "Authentication attempt budget exhausted"),
        (status = 503, description = "OIDC flow capacity unavailable")
    )
)]
pub async fn start_oidc_link(
    Extension(user_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
) -> HandlerResponse<impl IntoResponse> {
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
    let authorization_url = state
        .oidc_service()
        .start_authorization(mode)
        .await
        .map_err(|error| map_account_error(error, AccountMutation::Update))?;
    Ok(http_resp_sensitive(
        OidcAuthorizationResponse { authorization_url },
        (),
        start,
    ))
}
