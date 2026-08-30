//! Fixed-capacity synchronization for bundle publication and identity decoding.

use std::sync::Arc;

use tokio::sync::{
    OwnedRwLockReadGuard, OwnedRwLockWriteGuard, OwnedSemaphorePermit, RwLock, Semaphore,
};
use uuid::Uuid;

use super::super::error::WasmError;

const MODULE_LOCK_STRIPES: usize = 256;
const IDENTITY_DECOMPRESSION_JOBS: usize = 2;

/// Process-owned synchronization with no per-module allocation or growing key map.
pub struct WasmCoordination {
    module_locks: Box<[Arc<RwLock<()>>]>,
    identity_decompression: Arc<Semaphore>,
}

impl WasmCoordination {
    pub fn new() -> Self {
        Self::with_limits(MODULE_LOCK_STRIPES, IDENTITY_DECOMPRESSION_JOBS)
    }

    fn with_limits(module_lock_stripes: usize, identity_decompression_jobs: usize) -> Self {
        let module_lock_stripes = module_lock_stripes.max(1).next_power_of_two();
        let module_locks = (0..module_lock_stripes)
            .map(|_| Arc::new(RwLock::new(())))
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Self {
            module_locks,
            identity_decompression: Arc::new(Semaphore::new(identity_decompression_jobs.max(1))),
        }
    }

    pub async fn read_module(&self, module_id: Uuid) -> OwnedRwLockReadGuard<()> {
        self.module_lock(module_id).read_owned().await
    }

    pub async fn write_module(&self, module_id: Uuid) -> OwnedRwLockWriteGuard<()> {
        self.module_lock(module_id).write_owned().await
    }

    pub fn try_identity_decompression(&self) -> Result<OwnedSemaphorePermit, WasmError> {
        Arc::clone(&self.identity_decompression)
            .try_acquire_owned()
            .map_err(|_| WasmError::ServiceBusy)
    }

    fn module_lock(&self, module_id: Uuid) -> Arc<RwLock<()>> {
        let value = module_id.as_u128();
        let folded = (value as u64) ^ ((value >> 64) as u64);
        let index = (folded as usize) & (self.module_locks.len() - 1);
        Arc::clone(&self.module_locks[index])
    }
}

impl Default for WasmCoordination {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::WasmCoordination;
    use crate::features::wasm::error::WasmError;
    use uuid::Uuid;

    #[tokio::test]
    async fn module_writer_waits_for_an_existing_reader() {
        let coordination = WasmCoordination::with_limits(8, 1);
        let module_id = Uuid::now_v7();
        let reader = coordination.read_module(module_id).await;
        assert!(coordination.module_lock(module_id).try_write().is_err());
        drop(reader);
        assert!(coordination.module_lock(module_id).try_write().is_ok());
    }

    #[test]
    fn identity_decompression_fails_fast_at_capacity() -> anyhow::Result<()> {
        let coordination = WasmCoordination::with_limits(8, 1);
        let permit = coordination.try_identity_decompression()?;
        assert!(matches!(
            coordination.try_identity_decompression(),
            Err(WasmError::ServiceBusy)
        ));
        drop(permit);
        assert!(coordination.try_identity_decompression().is_ok());
        Ok(())
    }
}
