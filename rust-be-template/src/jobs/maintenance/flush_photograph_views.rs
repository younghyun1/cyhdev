//! Periodic flush of buffered photograph view counts to the database.
//!
//! Detail-page opens accumulate through the photography service's bounded view
//! port. The state method below is a temporary compatibility adapter.

use std::sync::Arc;

use tracing::error;

use crate::init::state::ServerState;

pub async fn flush_photograph_views(state: Arc<ServerState>) {
    match state.photography_service().flush_views().await {
        Ok(_) => {}
        Err(e) => {
            error!(error = ?e, "Failed to flush photograph view counts");
        }
    }
}
