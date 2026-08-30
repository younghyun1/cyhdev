//! WebAssembly module creation with ordered object-store compensation.

use chrono::Utc;
use tracing::info;
use uuid::Uuid;

use crate::util::media::persistence::{
    MediaWriteError, PersistedMedia, persist_media_objects,
};

use super::{
    asset_inputs::StagedWasmAssets,
    assets::{prepare_bundle, prepare_thumbnail},
    wasm_service::WasmService,
};
use super::super::{domain::module::{NewWasmModule, WasmModule}, error::WasmError};

impl WasmService {
    pub async fn create_module(
        &self,
        actor_user_id: Uuid,
        assets: StagedWasmAssets,
    ) -> Result<WasmModule, WasmError> {
        let bundle = match assets.bundle {
            Some(bundle) => bundle,
            None => return Err(invalid_upload("Missing bundle file")),
        };
        let thumbnail = match assets.thumbnail {
            Some(thumbnail) => thumbnail,
            None => return Err(invalid_upload("Missing thumbnail image")),
        };
        let title = required_text(assets.title, "title")?;
        let description = required_text(assets.description, "description")?;
        let normalized = prepare_bundle(bundle).await?;
        let bundle_kind = normalized.kind;
        let module_id = Uuid::now_v7();
        let prepared_thumbnail = prepare_thumbnail(
            module_id,
            thumbnail,
            self.object_store_region.as_ref(),
        )
        .await?;
        let pending = [prepared_thumbnail.pending.clone()];
        let thumbnail_url = prepared_thumbnail.url.clone();
        let repository = self.repository.clone();
        let result = persist_media_objects(self.object_store.as_ref(), &pending, async move {
            let now = Utc::now();
            let module = repository
                .insert_authorized(
                    actor_user_id,
                    NewWasmModule {
                        wasm_module_id: module_id,
                        user_id: actor_user_id,
                        wasm_module_link: format!("/api/wasm-modules/{module_id}/wasm"),
                        wasm_module_description: description,
                        wasm_module_created_at: now,
                        wasm_module_updated_at: now,
                        wasm_module_thumbnail_link: thumbnail_url,
                        wasm_module_title: title,
                        wasm_module_bundle_gz: normalized.gz_bytes,
                    },
                )
                .await?;
            Ok(PersistedMedia::new(module, Vec::new()))
        })
        .await;
        drop(prepared_thumbnail);

        let module = match result {
            Ok(success) => {
                self.record_unregistered_cleanup_failures(module_id, &success.cleanup_failures)
                    .await;
                success.value
            }
            Err(MediaWriteError::Upload {
                source,
                compensation_failures,
            }) => {
                self.record_unregistered_cleanup_failures(module_id, &compensation_failures)
                    .await;
                return Err(WasmError::ObjectStore(source));
            }
            Err(MediaWriteError::Persistence {
                source,
                compensation_failures,
            }) => {
                self.record_unregistered_cleanup_failures(module_id, &compensation_failures)
                    .await;
                return Err(source);
            }
        };
        self.cache_bundle(
            module_id,
            module.wasm_module_bundle_gz.clone(),
            bundle_kind,
        )
        .await;
        info!(wasm_module_id = %module_id, user_id = %actor_user_id, "WebAssembly module uploaded");
        Ok(module)
    }
}

fn required_text(value: Option<String>, field: &'static str) -> Result<String, WasmError> {
    match value.filter(|value| !value.trim().is_empty()) {
        Some(value) => Ok(value),
        None => Err(invalid_upload(match field {
            "title" => "Missing title field",
            "description" => "Missing description field",
            _ => "Missing required field",
        })),
    }
}

fn invalid_upload(message: &'static str) -> WasmError {
    WasmError::Bundle(anyhow::anyhow!(message))
}
