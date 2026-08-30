//! Scheduled bounded forum notification expiry.

use std::sync::Arc;

use crate::init::state::ServerState;

pub async fn prune_forum_notifications(state: Arc<ServerState>) {
    match state.forum_service().prune_notifications().await {
        Ok(report) if report.deleted > 0 => tracing::info!(
            deleted = report.deleted,
            remaining_expired = report.remaining_expired,
            "Pruned expired forum notifications"
        ),
        Ok(_) => {}
        Err(error) => tracing::error!(error = %error, "Forum notification pruning failed"),
    }
}
