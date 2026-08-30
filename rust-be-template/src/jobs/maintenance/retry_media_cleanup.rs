//! Bounded scheduler adapter for durable media cleanup.

use std::sync::Arc;

use chrono::Utc;
use tracing::{error, info, warn};

use crate::features::accounts::service::account_service::AccountService;

pub async fn retry_media_object_cleanup(account_service: Arc<AccountService>) {
    let report = match account_service.retry_media_object_cleanup(Utc::now()).await {
        Ok(report) => report,
        Err(source) => {
            error!(error = %source, "Failed to run durable media cleanup retry");
            return;
        }
    };
    if report.attempted == 0 {
        return;
    }
    if report.settlement.finalized < report.remote_deleted
        || report.settlement.failures_recorded < report.remote_failed
    {
        warn!(
            remote_deleted = report.remote_deleted,
            finalized = report.settlement.finalized,
            remote_failed = report.remote_failed,
            failures_recorded = report.settlement.failures_recorded,
            ledger_errors = report.settlement.ledger_errors,
            "Media cleanup ledger changed or failed during retry settlement"
        );
    }
    info!(
        attempted = report.attempted,
        completed = report.remote_deleted,
        failed = report.remote_failed,
        finalized = report.settlement.finalized,
        failures_recorded = report.settlement.failures_recorded,
        ledger_errors = report.settlement.ledger_errors,
        "Retried durable media cleanup"
    );
}
