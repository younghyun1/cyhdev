//! Periodic eviction of terminal/stuck photograph batch sessions.
//!
//! The feature service evicts terminal or stuck sessions and their temp dirs;
//! the state method below remains only as a scheduler compatibility adapter.

use std::sync::Arc;

use chrono::Utc;

use crate::init::state::ServerState;

pub async fn prune_photograph_batches(state: Arc<ServerState>) {
    let now = Utc::now();
    let evicted = state.photography_service().prune_batches(now).await;
    if evicted > 0 {
        tracing::info!(
            evicted_batches = evicted,
            "Pruned terminal/stuck photograph batch sessions"
        );
    }
}
