use std::sync::Arc;

use crate::init::state::ServerState;

pub async fn update_system_stats(state: Arc<ServerState>) {
    state.server_status_service().update_system_stats().await;
}
