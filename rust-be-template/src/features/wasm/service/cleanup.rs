//! Durable post-commit object cleanup for WebAssembly thumbnails.

use tracing::{error, info};
use uuid::Uuid;

use crate::util::media::{
    cleanup::{EnqueuedMediaCleanup, REASON_DELETED_WASM_THUMBNAIL, settle_durable_cleanup},
    persistence::{CleanupFailure, cleanup_committed_objects},
};

use super::wasm_service::WasmService;

pub const WASM_CLEANUP_CONCURRENCY: usize = 4;

pub struct CleanupOutcome {
    pub deleted_count: usize,
    pub failure_count: usize,
    pub remaining_count: usize,
    pub unresolved_count: usize,
}

impl WasmService {
    pub(super) async fn settle_cleanup(
        &self,
        module_id: Uuid,
        cleanup: EnqueuedMediaCleanup,
    ) -> CleanupOutcome {
        let total = cleanup.resolved.len() + cleanup.unresolved_count;
        let locations = cleanup
            .resolved
            .iter()
            .map(|cleanup| cleanup.location.clone())
            .collect();
        let (cleaned, failures) = cleanup_committed_objects(
            self.object_store.as_ref(),
            locations,
            WASM_CLEANUP_CONCURRENCY,
        )
        .await;
        let settlement =
            settle_durable_cleanup(&self.accounts, cleanup.resolved, &cleaned, &failures).await;
        log_cleanup_failures(module_id, &failures);
        CleanupOutcome {
            deleted_count: cleaned.len(),
            failure_count: failures.len() + settlement.ledger_errors,
            remaining_count: total.saturating_sub(settlement.finalized),
            unresolved_count: cleanup.unresolved_count,
        }
    }

    pub(super) async fn record_unregistered_cleanup_failures(
        &self,
        module_id: Uuid,
        failures: &[CleanupFailure],
    ) {
        log_cleanup_failures(module_id, failures);
        if failures.is_empty() {
            return;
        }
        match self
            .accounts
            .enqueue_media_cleanup_failures(module_id, REASON_DELETED_WASM_THUMBNAIL, failures)
            .await
        {
            Ok(report) => info!(
                wasm_module_id = %module_id,
                submitted = report.submitted,
                inserted = report.inserted,
                already_registered = report.already_registered,
                "Durably registered failed WebAssembly compensation cleanup"
            ),
            Err(source) => error!(
                wasm_module_id = %module_id,
                failure_count = failures.len(),
                error = %source,
                "Failed to durably register WebAssembly compensation cleanup"
            ),
        }
    }
}

pub(super) fn log_cleanup_failures(module_id: Uuid, failures: &[CleanupFailure]) {
    for failure in failures {
        error!(
            wasm_module_id = %module_id,
            key = %failure.location.key(),
            retryable = failure.is_retryable(),
            error = %failure.error,
            "WebAssembly thumbnail cleanup remains pending"
        );
    }
}
