//! Outermost layers shared by the API, static assets, and embedded applications.

use axum::{Router, middleware::from_fn};

use crate::{
    init::state::ServerState, routers::middleware::request_deadline::enforce_request_deadline,
};

/// Wraps the complete router so every route, including fallbacks, gets the policies.
pub(super) fn apply(router: Router, _state: &ServerState) -> anyhow::Result<Router> {
    Ok(router.layer(from_fn(enforce_request_deadline)))
}
