//! Scheduled bounded delivery of due account-retention notices.

use std::sync::Arc;

use crate::init::state::ServerState;

pub async fn send_retention_notifications(state: Arc<ServerState>) {
    match state
        .account_service()
        .send_due_retention_notifications()
        .await
    {
        Ok(report) => {
            if report.claimed > 0 {
                tracing::info!(
                    claimed = report.claimed,
                    delivered = report.delivered,
                    failed = report.failed,
                    skipped = report.skipped,
                    "Processed account-retention notifications"
                );
            }
        }
        Err(error) => {
            tracing::error!(error = %error, "Account-retention notification batch failed");
        }
    }
}
