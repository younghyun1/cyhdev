//! Bounded WebAssembly module reads.

use chrono::{DateTime, Utc};
use diesel::{BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    persistence::public_authors::load_public_authors,
    schema::wasm_module,
};

use super::{records::WasmModuleMetadataRecord, wasm_repository::WasmRepository};
use super::super::{domain::module::WasmModuleMetadata, error::WasmError};

const CACHE_SYNC_QUERY_ROWS: i64 = 1;
pub const MAX_COMPATIBILITY_MODULE_LIST: usize = 1_024;
const MAX_COMPATIBILITY_MODULE_QUERY_ROWS: i64 = 1_025;

pub struct WasmMetadataList {
    pub items: Vec<WasmModuleMetadata>,
    pub truncated: bool,
}

pub struct WasmBundlePageItem {
    pub module_id: Uuid,
    pub updated_at: DateTime<Utc>,
    pub gz_bytes: Vec<u8>,
}

impl WasmRepository {
    pub async fn list_metadata(&self) -> Result<WasmMetadataList, WasmError> {
        let mut connection = self.connection().await?;
        let records = wasm_module::table
            .select(WasmModuleMetadataRecord::as_select())
            .order((
                wasm_module::wasm_module_created_at.desc(),
                wasm_module::wasm_module_id.desc(),
            ))
            .limit(MAX_COMPATIBILITY_MODULE_QUERY_ROWS)
            .load::<WasmModuleMetadataRecord>(&mut connection)
            .await
            .map_err(WasmError::Database)?;
        let mut items = records
            .into_iter()
            .map(WasmModuleMetadata::from)
            .collect::<Vec<_>>();
        let owner_ids = items.iter().map(|item| item.user_id).collect::<Vec<_>>();
        let authors = load_public_authors(&mut connection, &owner_ids)
            .await
            .map_err(WasmError::Database)?;
        for item in &mut items {
            item.user_id = authors
                .get(&item.user_id)
                .map_or_else(Uuid::nil, |author| author.public_user_id());
        }
        let truncated = items.len() > MAX_COMPATIBILITY_MODULE_LIST;
        if truncated {
            items.truncate(MAX_COMPATIBILITY_MODULE_LIST);
        }
        Ok(WasmMetadataList { items, truncated })
    }

    pub async fn bundle_by_id(&self, module_id: Uuid) -> Result<Option<Vec<u8>>, WasmError> {
        let mut connection = self.connection().await?;
        wasm_module::table
            .filter(wasm_module::wasm_module_id.eq(module_id))
            .select(wasm_module::wasm_module_bundle_gz)
            .first::<Vec<u8>>(&mut connection)
            .await
            .optional()
            .map_err(WasmError::Database)
    }

    pub async fn bundle_page(
        &self,
        after: Option<(DateTime<Utc>, Uuid)>,
    ) -> Result<Vec<WasmBundlePageItem>, WasmError> {
        let mut connection = self.connection().await?;
        let mut query = wasm_module::table
            .select((
                wasm_module::wasm_module_id,
                wasm_module::wasm_module_updated_at,
                wasm_module::wasm_module_bundle_gz,
            ))
            .order((
                wasm_module::wasm_module_updated_at.desc(),
                wasm_module::wasm_module_id.desc(),
            ))
            .into_boxed();
        if let Some((updated_at, module_id)) = after {
            query = query.filter(
                wasm_module::wasm_module_updated_at
                    .lt(updated_at)
                    .or(wasm_module::wasm_module_updated_at
                        .eq(updated_at)
                        .and(wasm_module::wasm_module_id.lt(module_id))),
            );
        }
        query
            .limit(CACHE_SYNC_QUERY_ROWS)
            .load::<(Uuid, DateTime<Utc>, Vec<u8>)>(&mut connection)
            .await
            .map(|rows| {
                rows.into_iter()
                    .map(|(module_id, updated_at, gz_bytes)| WasmBundlePageItem {
                        module_id,
                        updated_at,
                        gz_bytes,
                    })
                    .collect()
            })
            .map_err(WasmError::Database)
    }
}
