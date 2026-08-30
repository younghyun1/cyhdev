//! Expiry cleanup for bounded process-local authentication throttles.

use std::sync::Arc;

use crate::init::state::ServerState;

pub async fn prune_auth_abuse(state: Arc<ServerState>) {
    let report = state.auth_abuse_service().prune_expired().await;
    tracing::debug!(
        ip_records_removed = report.ip_records_removed,
        identity_records_removed = report.identity_records_removed,
        ip_records_retained = report.ip_records_retained,
        identity_records_retained = report.identity_records_retained,
        "Pruned expired authentication throttle records"
    );
}
