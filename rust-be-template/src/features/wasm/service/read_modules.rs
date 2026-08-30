//! Metadata listing, bounded cache hydration, and bundle read-through.

use std::sync::Arc;

use tracing::{error, info};
use uuid::Uuid;

use super::{
    bundle_processing::sniff_kind_from_gzip_bytes,
    wasm_service::WasmService,
};
use super::super::{
    domain::{bundle::{CachedWasmBundle, ServedWasmBundle}, module::WasmModuleMetadata},
    error::WasmError,
};

const CACHE_SYNC_MAX_INSPECTED_ROWS: usize = 4_096;

impl WasmService {
    pub async fn list_modules(&self) -> Result<Vec<WasmModuleMetadata>, WasmError> {
        let list = self.repository.list_metadata().await?;
        if list.truncated {
            tracing::warn!(
                limit = super::super::repository::queries::MAX_COMPATIBILITY_MODULE_LIST,
                "WebAssembly compatibility list reached its fixed newest-first limit"
            );
        }
        Ok(list.items)
    }

    pub async fn synchronize_cache(&self) -> Result<usize, WasmError> {
        self.cache.clear().await;
        let mut cursor = None;
        let mut inspected = 0_usize;
        loop {
            if inspected >= CACHE_SYNC_MAX_INSPECTED_ROWS {
                tracing::warn!(
                    inspected,
                    max_inspected = CACHE_SYNC_MAX_INSPECTED_ROWS,
                    "Stopped WebAssembly cache synchronization at its inspection bound"
                );
                break;
            }
            let item = match self.repository.bundle_page(cursor).await?.into_iter().next() {
                Some(item) => item,
                None => break,
            };
            cursor = Some((item.updated_at, item.module_id));
            inspected = inspected.saturating_add(1);
            match cached_bundle(item.gz_bytes).await {
                Ok(bundle) => {
                    let retained = super::cache::WasmModuleCache::retained_bytes_for_payload(
                        bundle.bytes.len(),
                    );
                    if retained > self.cache.remaining_bytes() {
                        break;
                    }
                    self.admit_bundle(item.module_id, bundle).await;
                }
                Err(error) => {
                    error!(
                        wasm_module_id = %item.module_id,
                        error = %error,
                        "Skipped invalid WebAssembly bundle during cache synchronization"
                    );
                }
            }
        }
        Ok(self.cache.len())
    }

    pub async fn bundle(&self, module_id: Uuid) -> Result<Option<CachedWasmBundle>, WasmError> {
        let module_read = self.coordination.read_module(module_id).await;
        if let Some(bundle) = self.cache.get(&module_id).await {
            return Ok(Some(bundle));
        }
        drop(module_read);

        // Only a miss takes the exclusive stripe. The second cache check gives
        // one DB read-through per stripe while also serializing update/delete.
        let _module_fill = self.coordination.write_module(module_id).await;
        if let Some(bundle) = self.cache.get(&module_id).await {
            return Ok(Some(bundle));
        }
        let gz_bytes = match self.repository.bundle_by_id(module_id).await? {
            Some(gz_bytes) => gz_bytes,
            None => return Ok(None),
        };
        let bundle = cached_bundle(gz_bytes).await?;
        self.admit_bundle(module_id, bundle.clone()).await;
        Ok(Some(bundle))
    }

    pub async fn served_bundle(
        &self,
        module_id: Uuid,
        accepts_gzip: bool,
    ) -> Result<Option<ServedWasmBundle>, WasmError> {
        let bundle = match self.bundle(module_id).await? {
            Some(bundle) => bundle,
            None => return Ok(None),
        };
        let content_type = bundle.kind.content_type();
        if bundle.is_gzipped && !accepts_gzip {
            let decompression = self.coordination.try_identity_decompression()?;
            let compressed = bundle.bytes;
            let bytes = tokio::task::spawn_blocking(move || {
                let result = super::bundle_processing::gzip_decompress_limited(
                    &compressed,
                    super::bundle_processing::MAX_BUNDLE_SIZE_BYTES as usize,
                );
                drop(decompression);
                result
            })
            .await?
            .map_err(WasmError::Bundle)?;
            return Ok(Some(ServedWasmBundle {
                bytes: Arc::from(bytes.into_boxed_slice()),
                content_type,
                content_encoding_gzip: false,
            }));
        }
        Ok(Some(ServedWasmBundle {
            bytes: bundle.bytes,
            content_type,
            content_encoding_gzip: bundle.is_gzipped,
        }))
    }

    pub(super) async fn cache_bundle(
        &self,
        module_id: Uuid,
        gz_bytes: Vec<u8>,
        kind: super::super::domain::bundle::WasmBundleKind,
    ) {
        let bundle = CachedWasmBundle {
            bytes: Arc::from(gz_bytes.into_boxed_slice()),
            is_gzipped: true,
            kind,
        };
        self.admit_bundle(module_id, bundle).await;
    }

    pub(super) async fn invalidate_bundle(&self, module_id: Uuid) {
        let _ = self.cache.invalidate(&module_id).await;
    }

    async fn admit_bundle(&self, module_id: Uuid, bundle: CachedWasmBundle) {
        let size_bytes = bundle.bytes.len();
        let kind = bundle.kind;
        let admitted = self.cache.upsert(module_id, bundle).await;
        info!(
            wasm_module_id = %module_id,
            size_bytes,
            content_type = kind.content_type(),
            admitted,
            cache_used_bytes = self.cache.used_bytes(),
            cache_entries = self.cache.len(),
            cache_evictions = self.cache.evictions(),
            cache_rejections = self.cache.rejected_entries(),
            "Loaded WebAssembly bundle into cache"
        );
    }
}

async fn cached_bundle(gz_bytes: Vec<u8>) -> Result<CachedWasmBundle, WasmError> {
    let (kind, gz_bytes) = tokio::task::spawn_blocking(move || {
        let kind = sniff_kind_from_gzip_bytes(&gz_bytes)?;
        Ok::<_, anyhow::Error>((kind, gz_bytes))
    })
    .await?
    .map_err(WasmError::Bundle)?;
    Ok(CachedWasmBundle {
        bytes: Arc::from(gz_bytes.into_boxed_slice()),
        is_gzipped: true,
        kind,
    })
}
