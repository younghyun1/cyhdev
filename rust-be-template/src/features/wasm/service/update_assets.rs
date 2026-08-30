//! WebAssembly asset replacement with compensation and durable cleanup.

use chrono::Utc;
use std::sync::Arc;

use tracing::warn;
use uuid::Uuid;

use crate::util::media::persistence::{MediaWriteError, PersistedMedia, persist_media_objects};

use super::super::{
    domain::module::{WasmAssetUpdate, WasmModuleMetadata},
    error::WasmError,
};
use super::{
    asset_inputs::StagedWasmAssets,
    assets::{prepare_bundle, prepare_thumbnail},
    wasm_service::WasmService,
};

impl WasmService {
    pub async fn update_assets(
        &self,
        actor_user_id: Uuid,
        module_id: Uuid,
        assets: StagedWasmAssets,
    ) -> Result<WasmModuleMetadata, WasmError> {
        let title = nonempty(assets.title);
        let description = nonempty(assets.description);
        let normalized = match assets.bundle {
            Some(bundle) => Some(prepare_bundle(bundle).await?),
            None => None,
        };
        let new_kind = normalized.as_ref().map(|bundle| bundle.kind);
        let bundle_changed = new_kind.is_some();
        let prepared_thumbnail = match assets.thumbnail {
            Some(thumbnail) => Some(
                prepare_thumbnail(module_id, thumbnail, self.object_store_region.as_ref()).await?,
            ),
            None => None,
        };
        let pending = prepared_thumbnail
            .as_ref()
            .map(|thumbnail| vec![thumbnail.pending.clone()])
            .unwrap_or_default();
        let thumbnail_changed = prepared_thumbnail.is_some();
        let new_thumbnail_url = prepared_thumbnail
            .as_ref()
            .map(|thumbnail| thumbnail.url.clone());
        let new_bundle = normalized.map(|bundle| bundle.gz_bytes);
        let repository = self.repository.clone();
        let cache = self.cache.clone();
        let coordination = Arc::clone(&self.coordination);
        let result = persist_media_objects(self.object_store.as_ref(), &pending, async move {
            let publication = coordination.write_module(module_id).await;
            if bundle_changed {
                // Invalidate before the commit await. Cancellation can then leave
                // only a miss, which reads whichever version PostgreSQL committed.
                let _ = cache.invalidate(&module_id).await;
            }
            let (updated, cleanup) = repository
                .update_assets_authorized(
                    actor_user_id,
                    module_id,
                    WasmAssetUpdate {
                        wasm_module_title: title,
                        wasm_module_description: description,
                        wasm_module_thumbnail_link: new_thumbnail_url,
                        wasm_module_bundle_gz: new_bundle,
                        wasm_module_updated_at: Utc::now(),
                    },
                    thumbnail_changed,
                )
                .await?;
            // Durable cleanup is settled after cache publication so object-store
            // latency never extends the module's publication critical section.
            Ok(PersistedMedia::new(
                (updated, cleanup, publication),
                Vec::new(),
            ))
        })
        .await;
        drop(prepared_thumbnail);

        let (mut updated, cleanup, publication) = match result {
            Ok(success) => {
                if success.value.1.unresolved_count > 0 {
                    warn!(
                        wasm_module_id = %module_id,
                        unresolved = success.value.1.unresolved_count,
                        "Superseded WebAssembly thumbnail requires administrative resolution"
                    );
                }
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
        if let Some(kind) = new_kind {
            let gz_bytes = std::mem::take(&mut updated.wasm_module_bundle_gz);
            self.cache_bundle(module_id, gz_bytes, kind).await;
        }
        drop(publication);
        if !cleanup.resolved.is_empty() || cleanup.unresolved_count > 0 {
            let _ = self.settle_cleanup(module_id, cleanup).await;
        }
        Ok(WasmModuleMetadata::from(updated))
    }
}

fn nonempty(value: Option<String>) -> Option<String> {
    value.filter(|value| !value.trim().is_empty())
}
