use std::sync::Arc;

use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use futures_util::TryStreamExt;
use tracing::{error, info};
use uuid::Uuid;

use super::ServerState;
use crate::schema::wasm_module;
use crate::util::time::now::tokio_now;
use crate::util::wasm_bundle::sniff_content_type_from_gzip_bytes;

impl ServerState {
    pub async fn sync_wasm_module_cache(&self) -> anyhow::Result<usize> {
        let start = tokio_now();
        let mut conn = self.get_conn().await?;

        let mut rows = wasm_module::table
            .select((
                wasm_module::wasm_module_id,
                wasm_module::wasm_module_bundle_gz,
            ))
            .order((
                wasm_module::wasm_module_updated_at.asc(),
                wasm_module::wasm_module_id.asc(),
            ))
            .load_stream::<(Uuid, Vec<u8>)>(&mut conn)
            .await?;

        self.wasm_module_cache.clear().await;
        while let Some((wasm_module_id, gz_bytes)) = rows.try_next().await? {
            let _ = self
                .cache_wasm_module_from_gzip(wasm_module_id, gz_bytes)
                .await;
        }
        drop(rows);
        drop(conn);
        let cached = self.wasm_module_cache.len();

        info!(
            elapsed = ?start.elapsed(),
            entries_cached = cached,
            "Synchronized WASM module cache."
        );

        Ok(cached)
    }

    pub async fn upsert_wasm_module_cache(
        &self,
        wasm_module_id: Uuid,
        gz_bytes: Vec<u8>,
        content_type: &'static str,
    ) {
        let bytes: Arc<[u8]> = Arc::from(gz_bytes.into_boxed_slice());
        let size_bytes = bytes.len();
        let entry = (bytes, true, content_type);
        let admitted = self.wasm_module_cache.upsert(wasm_module_id, entry).await;
        if !admitted {
            info!(
                wasm_module_id = %wasm_module_id,
                size_bytes,
                "WASM module exceeds the cache byte budget; serving without admission"
            );
        }
    }

    async fn cache_wasm_module_from_gzip(
        &self,
        wasm_module_id: Uuid,
        gz_bytes: Vec<u8>,
    ) -> Option<(Arc<[u8]>, bool, &'static str)> {
        let sniff_result = tokio::task::spawn_blocking(move || {
            let content_type = sniff_content_type_from_gzip_bytes(&gz_bytes)?;
            Ok::<(&'static str, Vec<u8>), anyhow::Error>((content_type, gz_bytes))
        })
        .await;

        let (content_type, gz_bytes) = match sniff_result {
            Ok(Ok(result)) => result,
            Ok(Err(e)) => {
                error!(error = ?e, wasm_module_id = %wasm_module_id, "Failed to sniff WASM bundle content type");
                return None;
            }
            Err(e) => {
                error!(error = ?e, wasm_module_id = %wasm_module_id, "Failed to join WASM bundle sniff task");
                return None;
            }
        };

        let bytes: Arc<[u8]> = Arc::from(gz_bytes.into_boxed_slice());
        let entry = (bytes.clone(), true, content_type);

        let admitted = self
            .wasm_module_cache
            .upsert(wasm_module_id, entry.clone())
            .await;

        info!(
            wasm_module_id = %wasm_module_id,
            size_bytes = bytes.len(),
            is_gzipped = true,
            content_type = content_type,
            admitted,
            cache_used_bytes = self.wasm_module_cache.used_bytes(),
            cache_entries = self.wasm_module_cache.len(),
            cache_evictions = self.wasm_module_cache.evictions(),
            cache_rejections = self.wasm_module_cache.rejected_entries(),
            "Loaded WASM module bundle into cache"
        );

        Some(entry)
    }

    pub async fn get_wasm_module(
        &self,
        wasm_module_id: Uuid,
    ) -> Option<(Arc<[u8]>, bool, &'static str)> {
        if let Some(entry) = self.wasm_module_cache.get(&wasm_module_id).await {
            return Some(entry);
        }

        let mut conn = match self.get_conn().await {
            Ok(conn) => conn,
            Err(e) => {
                error!(error = ?e, "Failed to get DB connection for WASM bundle");
                return None;
            }
        };

        let row: Option<(Uuid, Vec<u8>)> = wasm_module::table
            .select((wasm_module::wasm_module_id, wasm_module::wasm_module_bundle_gz))
            .filter(wasm_module::wasm_module_id.eq(wasm_module_id))
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| {
                error!(error = ?e, wasm_module_id = %wasm_module_id, "Failed to load WASM module from DB");
                e
            })
            .ok()?;

        drop(conn);

        let (_, gz_bytes) = row?;
        let entry = self
            .cache_wasm_module_from_gzip(wasm_module_id, gz_bytes)
            .await?;

        Some(entry)
    }

    pub async fn invalidate_wasm_module(&self, wasm_module_id: Uuid) {
        let _ = self.wasm_module_cache.invalidate(&wasm_module_id).await;
    }
}
