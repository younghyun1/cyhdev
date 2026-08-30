//! WebAssembly deletion with durable thumbnail cleanup.

use tracing::info;
use uuid::Uuid;

use super::{cleanup::CleanupOutcome, wasm_service::WasmService};
use super::super::error::WasmError;

pub struct DeleteModuleOutcome {
    pub module_id: Uuid,
    pub cleanup: CleanupOutcome,
}

impl WasmService {
    pub async fn delete_module(
        &self,
        actor_user_id: Uuid,
        module_id: Uuid,
    ) -> Result<DeleteModuleOutcome, WasmError> {
        let publication = self.coordination.write_module(module_id).await;
        // A cancelled request may drop after PostgreSQL commits. Pre-invalidation
        // makes that state a safe read-through miss instead of stale publication.
        self.invalidate_bundle(module_id).await;
        let cleanup = self
            .repository
            .delete_authorized(actor_user_id, module_id)
            .await?;
        drop(publication);
        let cleanup = self.settle_cleanup(module_id, cleanup).await;
        info!(
            wasm_module_id = %module_id,
            user_id = %actor_user_id,
            "WebAssembly module deleted"
        );
        Ok(DeleteModuleOutcome { module_id, cleanup })
    }
}
