//! Replace WebAssembly module assets with post-commit object cleanup.

use std::sync::Arc;

use axum::{
    Extension,
    extract::{Multipart, Path, State},
};
use chrono::Utc;
use diesel::{AsChangeset, QueryDsl};
use diesel_async::{AsyncConnection, RunQueryDsl};
use tracing::{error, warn};
use uuid::Uuid;

use crate::{
    domain::wasm_module::wasm_module::WasmModule,
    dto::responses::{
        response_data::{Response as ApiResponse, http_resp},
        wasm_module::WasmModuleItem,
    },
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    init::state::ServerState,
    schema::wasm_module,
    util::{
        image::{
            image_variant::{CyhdevImageType, IMAGE_ENCODING_FORMAT},
            map_image_format_to_db_enum::map_image_format_to_str,
            process_uploaded_image_files::process_uploaded_image_files,
        },
        media::{
            object_store::{ObjectLocation, S3MediaObjectStore},
            persistence::{
                CleanupFailure, MediaWriteError, PendingMediaObject, PersistedMedia,
                persist_media_objects,
            },
        },
        s3::AWS_S3_BUCKET_NAME,
        time::now::tokio_now,
        wasm_bundle::{normalize_bundle_file, sniff_content_type_from_gzip_bytes},
    },
};

use super::asset_upload::{MAX_BUNDLE_SIZE_BYTES, WasmAssetUpload};

#[derive(AsChangeset, Default)]
#[diesel(table_name = wasm_module)]
struct WasmModuleAssetsChangeset {
    wasm_module_title: Option<String>,
    wasm_module_description: Option<String>,
    wasm_module_thumbnail_link: Option<String>,
    wasm_module_bundle_gz: Option<Vec<u8>>,
    wasm_module_updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, thiserror::Error)]
enum WasmUpdateError {
    #[error("database pool checkout failed: {0}")]
    Pool(#[source] anyhow::Error),
    #[error("WebAssembly module update failed: {0}")]
    Update(#[source] diesel::result::Error),
}

#[utoipa::path(
    post,
    path = "/api/wasm-modules/{wasm_module_id}/assets",
    tag = "wasm_module",
    params(("wasm_module_id" = Uuid, Path, description = "WASM module UUID")),
    request_body(content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "WASM module updated", body = WasmModuleItem),
        (status = 400, description = "Invalid upload payload", body = CodeErrorResp),
        (status = 401, description = "Unauthorized", body = CodeErrorResp),
        (status = 403, description = "Forbidden (not superuser)", body = CodeErrorResp),
        (status = 404, description = "WASM module not found", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn update_wasm_module_assets(
    Extension(_user_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    Path(module_id): Path<Uuid>,
    mut multipart: Multipart,
) -> HandlerResponse<ApiResponse<WasmModuleItem, ()>> {
    let start = tokio_now();
    let assets = WasmAssetUpload::read(&mut multipart).await?;
    let title = nonempty(assets.title);
    let description = nonempty(assets.description);

    let normalized = match assets.bundle {
        Some(bundle) => {
            let path = bundle.source.path().to_path_buf();
            let is_gzipped = bundle.is_gzipped;
            let is_html = bundle.is_html;
            let normalized = tokio::task::spawn_blocking(move || {
                normalize_bundle_file(
                    &path,
                    is_gzipped,
                    is_html,
                    MAX_BUNDLE_SIZE_BYTES as usize,
                )
            })
            .await
            .map_err(|source| code_err(CodeError::FILE_UPLOAD_ERROR, source))?
            .map_err(|source| code_err(CodeError::FILE_UPLOAD_ERROR, source))?;
            drop(bundle.source);
            Some(normalized)
        }
        None => None,
    };

    let (thumbnail_url, pending) = match assets.thumbnail {
        Some(thumbnail) => {
            let mut outputs = process_uploaded_image_files(
                thumbnail.path(),
                None,
                vec![CyhdevImageType::DemoThumbnail],
            )
            .await
            .map_err(|source| code_err(CodeError::COULD_NOT_PROCESS_IMAGE, source))?
            .into_iter();
            let output = outputs.next().ok_or_else(|| {
                code_err(
                    CodeError::COULD_NOT_PROCESS_IMAGE,
                    "Thumbnail encoder produced no output",
                )
            })?;
            drop(thumbnail);
            let asset_id = Uuid::now_v7();
            let (extension, _) = map_image_format_to_str(IMAGE_ENCODING_FORMAT);
            let location = ObjectLocation::new(
                AWS_S3_BUCKET_NAME,
                format!("wasm-thumbnails/{module_id}/{asset_id}.{extension}"),
            );
            let region = state
                .aws_profile_picture_config
                .region()
                .map(|region| region.to_string())
                .unwrap_or_else(|| "us-west-1".to_string());
            let url = location.public_s3_url(&region);
            let pending = vec![PendingMediaObject {
                location,
                content_type: "image/avif".to_string(),
                source: output.path_buf(),
            }];
            (Some(url), (pending, Some(output)))
        }
        None => (None, (Vec::new(), None)),
    };
    let (pending, _processed_thumbnail) = pending;
    let thumbnail_changed = thumbnail_url.is_some();
    let new_thumbnail_url = thumbnail_url.clone();
    let new_content_type = normalized.as_ref().map(|bundle| bundle.content_type);
    let new_bundle = normalized.map(|bundle| bundle.gz_bytes);
    let store = S3MediaObjectStore::from_config(&state.aws_profile_picture_config);
    let persistence_state = Arc::clone(&state);

    let result = persist_media_objects(&store, &pending, async move {
        let mut connection = persistence_state
            .get_conn()
            .await
            .map_err(WasmUpdateError::Pool)?;
        let (updated, old_thumbnail) = connection
            .transaction::<(WasmModule, String), diesel::result::Error, _>(async |connection| {
                let old_thumbnail = wasm_module::table
                    .find(module_id)
                    .select(wasm_module::wasm_module_thumbnail_link)
                    .for_update()
                    .first::<String>(&mut *connection)
                    .await?;
                let updated = diesel::update(wasm_module::table.find(module_id))
                    .set(WasmModuleAssetsChangeset {
                        wasm_module_title: title,
                        wasm_module_description: description,
                        wasm_module_thumbnail_link: new_thumbnail_url,
                        wasm_module_bundle_gz: new_bundle,
                        wasm_module_updated_at: Some(Utc::now()),
                    })
                    .get_result(&mut *connection)
                    .await?;
                Ok((updated, old_thumbnail))
            })
            .await
            .map_err(WasmUpdateError::Update)?;
        drop(connection);
        let superseded = if thumbnail_changed {
            match ObjectLocation::from_public_s3_url(AWS_S3_BUCKET_NAME, &old_thumbnail) {
                Some(location) => vec![location],
                None => {
                    warn!(wasm_module_id = %module_id, url = %old_thumbnail, "Skipped invalid old WASM thumbnail URL");
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };
        Ok(PersistedMedia::new(updated, superseded))
    })
    .await;

    let updated = match result {
        Ok(success) => {
            log_cleanup_failures(module_id, &success.cleanup_failures);
            success.value
        }
        Err(MediaWriteError::Upload {
            source,
            compensation_failures,
        }) => {
            log_cleanup_failures(module_id, &compensation_failures);
            return Err(code_err(CodeError::FILE_UPLOAD_ERROR, source));
        }
        Err(MediaWriteError::Persistence {
            source,
            compensation_failures,
        }) => {
            log_cleanup_failures(module_id, &compensation_failures);
            return Err(map_update_error(source));
        }
    };

    let content_type = match new_content_type {
        Some(content_type) => content_type,
        None => sniff_content_type_from_gzip_bytes(&updated.wasm_module_bundle_gz)
            .map_err(|source| code_err(CodeError::DB_UPDATE_ERROR, source))?,
    };
    state
        .upsert_wasm_module_cache(
            module_id,
            updated.wasm_module_bundle_gz.clone(),
            content_type,
        )
        .await;
    Ok(http_resp(WasmModuleItem::from(updated), (), start))
}

fn nonempty(value: Option<String>) -> Option<String> {
    value.filter(|value| !value.trim().is_empty())
}

fn map_update_error(source: WasmUpdateError) -> CodeErrorResp {
    match &source {
        WasmUpdateError::Pool(_) => code_err(CodeError::POOL_ERROR, source),
        WasmUpdateError::Update(diesel::result::Error::NotFound) => {
            code_err(CodeError::DB_QUERY_ERROR, "WASM module not found")
        }
        WasmUpdateError::Update(_) => code_err(CodeError::DB_UPDATE_ERROR, source),
    }
}

fn log_cleanup_failures(module_id: Uuid, failures: &[CleanupFailure]) {
    for failure in failures {
        error!(
            wasm_module_id = %module_id,
            key = %failure.location.key(),
            retryable = failure.is_retryable(),
            error = %failure.error,
            "WASM thumbnail cleanup remains pending"
        );
    }
}
