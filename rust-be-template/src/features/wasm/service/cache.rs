//! Byte-bounded process-local WebAssembly bundle cache.

use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicU64, AtomicUsize, Ordering},
    },
};

use scc::HashMap;
use tokio::sync::Mutex;
use uuid::Uuid;

use super::super::domain::bundle::CachedWasmBundle;

pub const WASM_MODULE_CACHE_MAX_BYTES: usize = 256 * 1024 * 1024;
const WASM_MODULE_ENTRY_OVERHEAD_BYTES: usize = 256;

struct CachedEntry {
    bundle: CachedWasmBundle,
    sequence: u64,
    retained_bytes: usize,
}

struct CacheInner {
    entries: HashMap<Uuid, CachedEntry>,
    eviction_order: Mutex<BTreeMap<u64, Uuid>>,
    used_bytes: AtomicUsize,
    sequence: AtomicU64,
    evictions: AtomicU64,
    rejected_entries: AtomicU64,
    max_bytes: usize,
}

/// Cloneable handle to the single bounded cache used by the backend process.
#[derive(Clone)]
pub struct WasmModuleCache {
    inner: Arc<CacheInner>,
}

impl WasmModuleCache {
    pub fn new(max_bytes: usize) -> Self {
        Self {
            inner: Arc::new(CacheInner {
                entries: HashMap::new(),
                eviction_order: Mutex::new(BTreeMap::new()),
                used_bytes: AtomicUsize::new(0),
                sequence: AtomicU64::new(0),
                evictions: AtomicU64::new(0),
                rejected_entries: AtomicU64::new(0),
                max_bytes,
            }),
        }
    }

    pub async fn upsert(&self, module_id: Uuid, bundle: CachedWasmBundle) -> bool {
        let mut eviction_order = self.inner.eviction_order.lock().await;
        let retained_bytes = Self::retained_bytes_for_payload(bundle.bytes.len());
        if retained_bytes > self.inner.max_bytes {
            let _ = self.invalidate_locked(&module_id, &mut eviction_order).await;
            self.inner
                .rejected_entries
                .fetch_add(1, Ordering::Relaxed);
            return false;
        }

        let sequence = self.inner.sequence.fetch_add(1, Ordering::Relaxed);
        let cached = CachedEntry {
            bundle,
            sequence,
            retained_bytes,
        };
        if let Some(previous) = self.inner.entries.upsert_async(module_id, cached).await {
            eviction_order.remove(&previous.sequence);
            self.inner
                .used_bytes
                .fetch_sub(previous.retained_bytes, Ordering::SeqCst);
        }
        self.inner
            .used_bytes
            .fetch_add(retained_bytes, Ordering::SeqCst);
        eviction_order.insert(sequence, module_id);
        self.evict_over_budget(&mut eviction_order).await;
        true
    }

    pub async fn get(&self, module_id: &Uuid) -> Option<CachedWasmBundle> {
        self.inner
            .entries
            .read_async(module_id, |_, cached| cached.bundle.clone())
            .await
    }

    pub async fn invalidate(&self, module_id: &Uuid) -> bool {
        let mut eviction_order = self.inner.eviction_order.lock().await;
        self.invalidate_locked(module_id, &mut eviction_order).await
    }

    async fn invalidate_locked(
        &self,
        module_id: &Uuid,
        eviction_order: &mut BTreeMap<u64, Uuid>,
    ) -> bool {
        match self.inner.entries.remove_async(module_id).await {
            Some((_, cached)) => {
                eviction_order.remove(&cached.sequence);
                self.inner
                    .used_bytes
                    .fetch_sub(cached.retained_bytes, Ordering::SeqCst);
                true
            }
            None => false,
        }
    }

    pub async fn clear(&self) {
        let mut eviction_order = self.inner.eviction_order.lock().await;
        self.inner.entries.clear_async().await;
        eviction_order.clear();
        self.inner.used_bytes.store(0, Ordering::SeqCst);
    }

    pub fn len(&self) -> usize {
        self.inner.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.entries.is_empty()
    }

    pub fn used_bytes(&self) -> usize {
        self.inner.used_bytes.load(Ordering::Relaxed)
    }

    pub fn remaining_bytes(&self) -> usize {
        self.inner.max_bytes.saturating_sub(self.used_bytes())
    }

    pub fn retained_bytes_for_payload(payload_bytes: usize) -> usize {
        payload_bytes.saturating_add(WASM_MODULE_ENTRY_OVERHEAD_BYTES)
    }

    pub fn evictions(&self) -> u64 {
        self.inner.evictions.load(Ordering::Relaxed)
    }

    pub fn rejected_entries(&self) -> u64 {
        self.inner.rejected_entries.load(Ordering::Relaxed)
    }

    async fn evict_over_budget(&self, eviction_order: &mut BTreeMap<u64, Uuid>) {
        while self.inner.used_bytes.load(Ordering::SeqCst) > self.inner.max_bytes {
            let (sequence, module_id) = match eviction_order.pop_first() {
                Some(queued) => queued,
                None => return,
            };
            if let Some((_, removed)) = self
                .inner
                .entries
                .remove_if_async(&module_id, |cached| cached.sequence == sequence)
                .await
            {
                self.inner
                    .used_bytes
                    .fetch_sub(removed.retained_bytes, Ordering::SeqCst);
                self.inner.evictions.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

impl Default for WasmModuleCache {
    fn default() -> Self {
        Self::new(WASM_MODULE_CACHE_MAX_BYTES)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::wasm::domain::bundle::WasmBundleKind;

    fn bundle(bytes: usize) -> CachedWasmBundle {
        CachedWasmBundle {
            bytes: Arc::from(vec![0_u8; bytes].into_boxed_slice()),
            is_gzipped: true,
            kind: WasmBundleKind::WebAssembly,
        }
    }

    #[tokio::test]
    async fn cache_evicts_oldest_live_entry_to_restore_its_byte_bound() {
        let cache = WasmModuleCache::new(600);
        let first = Uuid::now_v7();
        let second = Uuid::now_v7();
        assert!(cache.upsert(first, bundle(100)).await);
        assert!(cache.upsert(second, bundle(100)).await);
        assert!(cache.get(&first).await.is_none());
        assert!(cache.get(&second).await.is_some());
        assert_eq!(cache.len(), 1);
        assert_eq!(cache.evictions(), 1);
        assert!(cache.used_bytes() <= 600);
    }

    #[tokio::test]
    async fn oversized_replacement_is_rejected_and_invalidates_stale_bytes() {
        let cache = WasmModuleCache::new(400);
        let module_id = Uuid::now_v7();
        assert!(cache.upsert(module_id, bundle(100)).await);
        assert!(!cache.upsert(module_id, bundle(401)).await);
        assert!(cache.get(&module_id).await.is_none());
        assert_eq!(cache.rejected_entries(), 1);
    }

    #[tokio::test]
    async fn remaining_budget_includes_fixed_entry_overhead() {
        let cache = WasmModuleCache::new(600);
        let module_id = Uuid::now_v7();
        assert!(cache.upsert(module_id, bundle(100)).await);
        assert_eq!(cache.remaining_bytes(), 244);
        assert_eq!(WasmModuleCache::retained_bytes_for_payload(100), 356);
    }
}
