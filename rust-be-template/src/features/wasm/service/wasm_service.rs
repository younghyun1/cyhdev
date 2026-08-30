//! Explicit dependencies for WebAssembly use cases.

use std::sync::Arc;

use crate::{
    features::accounts::service::account_service::AccountService,
    util::media::object_store::MediaObjectStore,
};

use super::super::repository::wasm_repository::WasmRepository;
use super::{cache::WasmModuleCache, coordination::WasmCoordination};

#[derive(Clone)]
pub struct WasmService {
    pub(super) repository: Arc<WasmRepository>,
    pub(super) cache: WasmModuleCache,
    pub(super) object_store: Arc<dyn MediaObjectStore>,
    pub(super) object_store_region: Arc<str>,
    pub(super) accounts: Arc<AccountService>,
    pub(super) coordination: Arc<WasmCoordination>,
}

impl WasmService {
    pub fn new(
        repository: Arc<WasmRepository>,
        cache: WasmModuleCache,
        object_store: Arc<dyn MediaObjectStore>,
        object_store_region: impl Into<Arc<str>>,
        accounts: Arc<AccountService>,
    ) -> Self {
        Self {
            repository,
            cache,
            object_store,
            object_store_region: object_store_region.into(),
            accounts,
            coordination: Arc::new(WasmCoordination::new()),
        }
    }
}
