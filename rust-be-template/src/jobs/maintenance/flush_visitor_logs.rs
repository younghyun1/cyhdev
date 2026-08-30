use std::sync::Arc;

use tracing::error;

use crate::init::state::ServerState;

pub async fn flush_visitor_logs(state: Arc<ServerState>) {
    match state.visitor_service().flush().await {
        Ok(_) => {}
        Err(e) => {
            error!(error = ?e, "Failed to flush buffered visitor logs");
        }
    }
}
