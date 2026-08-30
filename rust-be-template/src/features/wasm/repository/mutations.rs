//! Transactional WebAssembly writes and durable cleanup enqueueing.

use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    features::accounts::{
        domain::role::RoleType,
    },
    persistence::media_cleanup::enqueue_media_cleanup,
    schema::{user_roles, users, wasm_module},
    util::media::cleanup::{
        EnqueuedMediaCleanup, MediaCleanupRequest, REASON_DELETED_WASM_THUMBNAIL,
        REASON_SUPERSEDED_WASM_THUMBNAIL,
    },
};

use super::{
    records::{
        NewWasmModuleRecord, WasmAssetChangeset, WasmMetadataChangeset, WasmModuleMetadataRecord,
        WasmModuleRecord,
    },
    wasm_repository::WasmRepository,
};
use super::super::{
    domain::module::{
        NewWasmModule, WasmAssetUpdate, WasmMetadataUpdate, WasmModule, WasmModuleMetadata,
    },
    error::WasmError,
};

impl WasmRepository {
    pub async fn insert_authorized(
        &self,
        actor_user_id: Uuid,
        module: NewWasmModule,
    ) -> Result<WasmModule, WasmError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<WasmModule, WasmError, _>(async move |connection| {
                lock_active_superuser(connection, actor_user_id).await?;
                diesel::insert_into(wasm_module::table)
                    .values(NewWasmModuleRecord::from(module))
                    .returning(WasmModuleRecord::as_returning())
                    .get_result::<WasmModuleRecord>(&mut *connection)
                    .await
                    .map(WasmModule::from)
                    .map_err(WasmError::Database)
            })
            .await
    }

    pub async fn update_metadata_authorized(
        &self,
        actor_user_id: Uuid,
        module_id: Uuid,
        update: WasmMetadataUpdate,
    ) -> Result<WasmModuleMetadata, WasmError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<WasmModuleMetadata, WasmError, _>(async move |connection| {
                lock_active_superuser(connection, actor_user_id).await?;
                diesel::update(wasm_module::table.find(module_id))
                    .set(WasmMetadataChangeset::from(update))
                    .returning(WasmModuleMetadataRecord::as_returning())
                    .get_result::<WasmModuleMetadataRecord>(&mut *connection)
                    .await
                    .optional()?
                    .map(WasmModuleMetadata::from)
                    .ok_or(WasmError::NotFound)
            })
            .await
    }

    pub async fn update_assets_authorized(
        &self,
        actor_user_id: Uuid,
        module_id: Uuid,
        update: WasmAssetUpdate,
        thumbnail_changed: bool,
    ) -> Result<(WasmModule, EnqueuedMediaCleanup), WasmError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<(WasmModule, EnqueuedMediaCleanup), WasmError, _>(
                async move |connection| {
                    lock_active_superuser(connection, actor_user_id).await?;
                    let old_thumbnail = wasm_module::table
                        .find(module_id)
                        .select(wasm_module::wasm_module_thumbnail_link)
                        .for_update()
                        .first::<String>(&mut *connection)
                        .await
                        .optional()?
                        .ok_or(WasmError::NotFound)?;
                    let cleanup = if thumbnail_changed {
                        enqueue_media_cleanup(
                            connection,
                            vec![MediaCleanupRequest {
                                original_url: old_thumbnail,
                                reason: REASON_SUPERSEDED_WASM_THUMBNAIL,
                                source_id: module_id,
                            }],
                        )
                        .await?
                    } else {
                        EnqueuedMediaCleanup {
                            resolved: Vec::new(),
                            unresolved_count: 0,
                        }
                    };
                    let updated = diesel::update(wasm_module::table.find(module_id))
                        .set(WasmAssetChangeset::from(update))
                        .returning(WasmModuleRecord::as_returning())
                        .get_result::<WasmModuleRecord>(&mut *connection)
                        .await
                        .optional()?
                        .map(WasmModule::from)
                        .ok_or(WasmError::NotFound)?;
                    Ok((updated, cleanup))
                },
            )
            .await
    }

    pub async fn delete_authorized(
        &self,
        actor_user_id: Uuid,
        module_id: Uuid,
    ) -> Result<EnqueuedMediaCleanup, WasmError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<EnqueuedMediaCleanup, WasmError, _>(async move |connection| {
                lock_active_superuser(connection, actor_user_id).await?;
                let thumbnail_url = wasm_module::table
                    .find(module_id)
                    .select(wasm_module::wasm_module_thumbnail_link)
                    .for_update()
                    .first::<String>(&mut *connection)
                    .await
                    .optional()?
                    .ok_or(WasmError::NotFound)?;
                let cleanup = enqueue_media_cleanup(
                    connection,
                    vec![MediaCleanupRequest {
                        original_url: thumbnail_url,
                        reason: REASON_DELETED_WASM_THUMBNAIL,
                        source_id: module_id,
                    }],
                )
                .await?;
                let deleted = diesel::delete(wasm_module::table.find(module_id))
                    .execute(&mut *connection)
                    .await?;
                if deleted == 1 {
                    Ok(cleanup)
                } else {
                    Err(WasmError::NotFound)
                }
            })
            .await
    }
}

async fn lock_active_superuser(
    connection: &mut AsyncPgConnection,
    user_id: Uuid,
) -> Result<(), WasmError> {
    let active = users::table
        .filter(users::user_id.eq(user_id))
        .filter(users::user_deleted_at.is_null())
        .filter(users::user_hard_purged_at.is_null())
        .filter(users::user_is_email_verified.eq(true))
        .filter(users::user_is_system_actor.eq(false))
        .select(users::user_id)
        .for_update()
        .first::<Uuid>(&mut *connection)
        .await
        .optional()?;
    if active.is_none() {
        return Err(WasmError::Unauthorized);
    }
    let role_id = user_roles::table
        .filter(user_roles::user_id.eq(user_id))
        .select(user_roles::role_id)
        .for_update()
        .first::<Uuid>(&mut *connection)
        .await
        .optional()?;
    match role_id.and_then(RoleType::from_uuid) {
        Some(role) if role.is_superuser() => Ok(()),
        Some(_) | None => Err(WasmError::Unauthorized),
    }
}
