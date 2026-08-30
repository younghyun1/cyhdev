//! WebAssembly module domain records and commands.

use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct WasmModule {
    pub wasm_module_id: Uuid,
    pub user_id: Uuid,
    pub wasm_module_link: String,
    pub wasm_module_description: String,
    pub wasm_module_created_at: DateTime<Utc>,
    pub wasm_module_updated_at: DateTime<Utc>,
    pub wasm_module_thumbnail_link: String,
    pub wasm_module_title: String,
    pub wasm_module_bundle_gz: Vec<u8>,
}

#[derive(Clone)]
pub struct WasmModuleMetadata {
    pub wasm_module_id: Uuid,
    pub user_id: Uuid,
    pub wasm_module_link: String,
    pub wasm_module_description: String,
    pub wasm_module_created_at: DateTime<Utc>,
    pub wasm_module_updated_at: DateTime<Utc>,
    pub wasm_module_thumbnail_link: String,
    pub wasm_module_title: String,
}

pub struct NewWasmModule {
    pub wasm_module_id: Uuid,
    pub user_id: Uuid,
    pub wasm_module_link: String,
    pub wasm_module_description: String,
    pub wasm_module_created_at: DateTime<Utc>,
    pub wasm_module_updated_at: DateTime<Utc>,
    pub wasm_module_thumbnail_link: String,
    pub wasm_module_title: String,
    pub wasm_module_bundle_gz: Vec<u8>,
}

pub struct WasmMetadataUpdate {
    pub wasm_module_title: Option<String>,
    pub wasm_module_description: Option<String>,
    pub wasm_module_updated_at: DateTime<Utc>,
}

pub struct WasmAssetUpdate {
    pub wasm_module_title: Option<String>,
    pub wasm_module_description: Option<String>,
    pub wasm_module_thumbnail_link: Option<String>,
    pub wasm_module_bundle_gz: Option<Vec<u8>>,
    pub wasm_module_updated_at: DateTime<Utc>,
}

impl From<WasmModule> for WasmModuleMetadata {
    fn from(module: WasmModule) -> Self {
        Self {
            wasm_module_id: module.wasm_module_id,
            user_id: module.user_id,
            wasm_module_link: module.wasm_module_link,
            wasm_module_description: module.wasm_module_description,
            wasm_module_created_at: module.wasm_module_created_at,
            wasm_module_updated_at: module.wasm_module_updated_at,
            wasm_module_thumbnail_link: module.wasm_module_thumbnail_link,
            wasm_module_title: module.wasm_module_title,
        }
    }
}
