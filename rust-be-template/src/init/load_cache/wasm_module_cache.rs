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

/// Maximum approximate retained bytes for compressed WASM bundles and metadata.
pub const WASM_MODULE_CACHE_MAX_BYTES: usize = 256 * 1024 * 1024;
const WASM_MODULE_ENTRY_OVERHEAD_BYTES: usize = 256;

pub type WasmModuleEntry = (Arc<[u8]>, bool, &'static str);

struct CachedWasmModule {
    entry: WasmModuleEntry,
    sequence: u64,
    retained_bytes: usize,
}

/// Byte-bounded FIFO cache for compressed WASM bundles.
///
/// PostgreSQL remains authoritative. Entries larger than the whole budget are
/// served without admission; otherwise the oldest admitted live entries are
/// evicted until the cache is within budget. Sequence numbers make stale FIFO
/// records from an upsert harmless.
pub struct WasmModuleCache {
    entries: HashMap<Uuid, CachedWasmModule>,
    eviction_order: Mutex<BTreeMap<u64, Uuid>>,
    used_bytes: AtomicUsize,
    sequence: AtomicU64,
    evictions: AtomicU64,
    rejected_entries: AtomicU64,
    max_bytes: usize,
}

impl WasmModuleCache {
    pub fn new(max_bytes: usize) -> Self {
        Self {
            entries: HashMap::new(),
            eviction_order: Mutex::new(BTreeMap::new()),
            used_bytes: AtomicUsize::new(0),
            sequence: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
            rejected_entries: AtomicU64::new(0),
            max_bytes,
        }
    }

    /// Admit an entry and evict the oldest live entries to restore the budget.
    /// Returns false when the entry itself is larger than the entire cache.
    pub async fn upsert(&self, module_id: Uuid, entry: WasmModuleEntry) -> bool {
        let mut eviction_order = self.eviction_order.lock().await;
        let retained_bytes = entry
            .0
            .len()
            .saturating_add(WASM_MODULE_ENTRY_OVERHEAD_BYTES);
        if retained_bytes > self.max_bytes {
            let _ = self
                .invalidate_unlocked(&module_id, &mut eviction_order)
                .await;
            self.rejected_entries.fetch_add(1, Ordering::Relaxed);
            return false;
        }

        let sequence = self.sequence.fetch_add(1, Ordering::Relaxed);
        let cached = CachedWasmModule {
            entry,
            sequence,
            retained_bytes,
        };
        if let Some(previous) = self.entries.upsert_async(module_id, cached).await {
            eviction_order.remove(&previous.sequence);
            self.used_bytes
                .fetch_sub(previous.retained_bytes, Ordering::SeqCst);
        }
        self.used_bytes
            .fetch_add(retained_bytes, Ordering::SeqCst);
        eviction_order.insert(sequence, module_id);
        self.evict_over_budget(&mut eviction_order).await;
        true
    }

    pub async fn get(&self, module_id: &Uuid) -> Option<WasmModuleEntry> {
        self.entries
            .read_async(module_id, |_, cached| cached.entry.clone())
            .await
    }

    pub async fn invalidate(&self, module_id: &Uuid) -> bool {
        let mut eviction_order = self.eviction_order.lock().await;
        self.invalidate_unlocked(module_id, &mut eviction_order)
            .await
    }

    async fn invalidate_unlocked(
        &self,
        module_id: &Uuid,
        eviction_order: &mut BTreeMap<u64, Uuid>,
    ) -> bool {
        match self.entries.remove_async(module_id).await {
            Some((_, cached)) => {
                eviction_order.remove(&cached.sequence);
                self.used_bytes
                    .fetch_sub(cached.retained_bytes, Ordering::SeqCst);
                true
            }
            None => false,
        }
    }

    pub async fn clear(&self) {
        let mut eviction_order = self.eviction_order.lock().await;
        self.entries.clear_async().await;
        eviction_order.clear();
        self.used_bytes.store(0, Ordering::SeqCst);
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn used_bytes(&self) -> usize {
        self.used_bytes.load(Ordering::Relaxed)
    }

    pub fn evictions(&self) -> u64 {
        self.evictions.load(Ordering::Relaxed)
    }

    pub fn rejected_entries(&self) -> u64 {
        self.rejected_entries.load(Ordering::Relaxed)
    }

    async fn evict_over_budget(&self, eviction_order: &mut BTreeMap<u64, Uuid>) {
        while self.used_bytes.load(Ordering::SeqCst) > self.max_bytes {
            let (sequence, module_id) = match eviction_order.pop_first() {
                Some(queued) => queued,
                None => return,
            };
            if let Some((_, removed)) = self
                .entries
                .remove_if_async(&module_id, |cached| cached.sequence == sequence)
                .await
            {
                self.used_bytes
                    .fetch_sub(removed.retained_bytes, Ordering::SeqCst);
                self.evictions.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

impl Default for WasmModuleCache {
    fn default() -> Self {
        Self::new(WASM_MODULE_CACHE_MAX_BYTES)
    }
}
