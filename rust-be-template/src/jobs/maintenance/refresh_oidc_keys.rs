//! Hourly OpenID Connect signing-key refresh so provider key rotation needs no restart.

use std::sync::Arc;

use crate::init::state::ServerState;

pub async fn refresh_oidc_keys(state: Arc<ServerState>) {
    state.oidc_service().refresh_signing_keys().await;
}
