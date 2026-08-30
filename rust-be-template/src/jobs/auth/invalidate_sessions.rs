use tracing::info;

use crate::init::state::ServerState;
use std::sync::Arc;

pub async fn invalidate_sessions(state: Arc<ServerState>) {
    let (pruned, remaining): (usize, usize) = state.session_service().purge_expired().await;
    info!(
        pruned = pruned,
        remaining = remaining,
        "Invalidated expired sessions."
    );
}
