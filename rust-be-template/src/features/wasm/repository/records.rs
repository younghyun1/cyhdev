//! Diesel records isolated from the WebAssembly domain.

use chrono::{DateTime, Utc};
use diesel::{AsChangeset, Insertable, Queryable, Selectable};
use uuid::Uuid;

use crate::schema::wasm_module;

use super::super::domain::module::{
    NewWasmModule, WasmAssetUpdate, WasmMetadataUpdate, WasmModule, WasmModuleMetadata,
};

#[derive(Queryable, Selectable)]
#[diesel(table_name = wasm_module)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(super) struct WasmModuleRecord {
    wasm_module_id: Uuid,
    user_id: Uuid,
    wasm_module_link: String,
    wasm_module_description: String,
    wasm_module_created_at: DateTime<Utc>,
    wasm_module_updated_at: DateTime<Utc>,
    wasm_module_thumbnail_link: String,
    wasm_module_title: String,
    wasm_module_bundle_gz: Vec<u8>,
}

impl From<WasmModuleRecord> for WasmModule {
    fn from(record: WasmModuleRecord) -> Self {
        Self {
            wasm_module_id: record.wasm_module_id,
            user_id: record.user_id,
            wasm_module_link: record.wasm_module_link,
            wasm_module_description: record.wasm_module_description,
            wasm_module_created_at: record.wasm_module_created_at,
            wasm_module_updated_at: record.wasm_module_updated_at,
            wasm_module_thumbnail_link: record.wasm_module_thumbnail_link,
            wasm_module_title: record.wasm_module_title,
            wasm_module_bundle_gz: record.wasm_module_bundle_gz,
        }
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = wasm_module)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(super) struct WasmModuleMetadataRecord {
    wasm_module_id: Uuid,
    user_id: Uuid,
    wasm_module_link: String,
    wasm_module_description: String,
    wasm_module_created_at: DateTime<Utc>,
    wasm_module_updated_at: DateTime<Utc>,
    wasm_module_thumbnail_link: String,
    wasm_module_title: String,
}

impl From<WasmModuleMetadataRecord> for WasmModuleMetadata {
    fn from(record: WasmModuleMetadataRecord) -> Self {
        Self {
            wasm_module_id: record.wasm_module_id,
            user_id: record.user_id,
            wasm_module_link: record.wasm_module_link,
            wasm_module_description: record.wasm_module_description,
            wasm_module_created_at: record.wasm_module_created_at,
            wasm_module_updated_at: record.wasm_module_updated_at,
            wasm_module_thumbnail_link: record.wasm_module_thumbnail_link,
            wasm_module_title: record.wasm_module_title,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = wasm_module)]
pub(super) struct NewWasmModuleRecord {
    wasm_module_id: Uuid,
    user_id: Uuid,
    wasm_module_link: String,
    wasm_module_description: String,
    wasm_module_created_at: DateTime<Utc>,
    wasm_module_updated_at: DateTime<Utc>,
    wasm_module_thumbnail_link: String,
    wasm_module_title: String,
    wasm_module_bundle_gz: Vec<u8>,
}

impl From<NewWasmModule> for NewWasmModuleRecord {
    fn from(module: NewWasmModule) -> Self {
        Self {
            wasm_module_id: module.wasm_module_id,
            user_id: module.user_id,
            wasm_module_link: module.wasm_module_link,
            wasm_module_description: module.wasm_module_description,
            wasm_module_created_at: module.wasm_module_created_at,
            wasm_module_updated_at: module.wasm_module_updated_at,
            wasm_module_thumbnail_link: module.wasm_module_thumbnail_link,
            wasm_module_title: module.wasm_module_title,
            wasm_module_bundle_gz: module.wasm_module_bundle_gz,
        }
    }
}

#[derive(AsChangeset)]
#[diesel(table_name = wasm_module)]
pub(super) struct WasmMetadataChangeset {
    wasm_module_title: Option<String>,
    wasm_module_description: Option<String>,
    wasm_module_updated_at: DateTime<Utc>,
}

impl From<WasmMetadataUpdate> for WasmMetadataChangeset {
    fn from(update: WasmMetadataUpdate) -> Self {
        Self {
            wasm_module_title: update.wasm_module_title,
            wasm_module_description: update.wasm_module_description,
            wasm_module_updated_at: update.wasm_module_updated_at,
        }
    }
}

#[derive(AsChangeset)]
#[diesel(table_name = wasm_module)]
pub(super) struct WasmAssetChangeset {
    wasm_module_title: Option<String>,
    wasm_module_description: Option<String>,
    wasm_module_thumbnail_link: Option<String>,
    wasm_module_bundle_gz: Option<Vec<u8>>,
    wasm_module_updated_at: DateTime<Utc>,
}

impl From<WasmAssetUpdate> for WasmAssetChangeset {
    fn from(update: WasmAssetUpdate) -> Self {
        Self {
            wasm_module_title: update.wasm_module_title,
            wasm_module_description: update.wasm_module_description,
            wasm_module_thumbnail_link: update.wasm_module_thumbnail_link,
            wasm_module_bundle_gz: update.wasm_module_bundle_gz,
            wasm_module_updated_at: update.wasm_module_updated_at,
        }
    }
}
