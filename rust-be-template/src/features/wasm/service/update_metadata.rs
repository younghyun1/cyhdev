//! WebAssembly metadata update use case.

use chrono::Utc;
use tracing::info;
use uuid::Uuid;

use super::super::{
    domain::module::{WasmMetadataUpdate, WasmModuleMetadata},
    error::WasmError,
};
use super::wasm_service::WasmService;

impl WasmService {
    pub async fn update_metadata(
        &self,
        actor_user_id: Uuid,
        module_id: Uuid,
        title: Option<String>,
        description: Option<String>,
    ) -> Result<WasmModuleMetadata, WasmError> {
        let updated = self
            .repository
            .update_metadata_authorized(
                actor_user_id,
                module_id,
                WasmMetadataUpdate {
                    wasm_module_title: title,
                    wasm_module_description: description,
                    wasm_module_updated_at: Utc::now(),
                },
            )
            .await?;
        info!(
            wasm_module_id = %module_id,
            user_id = %actor_user_id,
            "WebAssembly module metadata updated"
        );
        Ok(updated)
    }
}
