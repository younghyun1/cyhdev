//! Create a WebAssembly module from bounded, staged multipart assets.

use std::sync::Arc;

use axum::{
    Extension,
    extract::{Multipart, State},
};
use chrono::Utc;
use diesel_async::RunQueryDsl;
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    domain::wasm_module::wasm_module::{WasmModule, WasmModuleInsertable},
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
        wasm_bundle::normalize_bundle_file,
    },
};

use super::asset_upload::{MAX_BUNDLE_SIZE_BYTES, WasmAssetUpload};

#[derive(Debug, thiserror::Error)]
enum WasmPersistenceError {
    #[error("database pool checkout failed: {0}")]
    Pool(#[source] anyhow::Error),
    #[error("WebAssembly module insert failed: {0}")]
    Insert(#[source] diesel::result::Error),
}

#[utoipa::path(
    post,
    path = "/api/wasm-modules",
    tag = "wasm_module",
    request_body(content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "WASM module uploaded successfully", body = WasmModuleItem),
        (status = 400, description = "Invalid upload payload", body = CodeErrorResp),
        (status = 401, description = "Unauthorized", body = CodeErrorResp),
        (status = 403, description = "Forbidden (not superuser)", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn upload_wasm_module(
    Extension(user_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    mut multipart: Multipart,
) -> HandlerResponse<ApiResponse<WasmModuleItem, ()>> {
    let start = tokio_now();
    let assets = WasmAssetUpload::read(&mut multipart).await?;
    let bundle = assets
        .bundle
        .ok_or_else(|| code_err(CodeError::FILE_UPLOAD_ERROR, "Missing bundle file"))?;
    let thumbnail = assets
        .thumbnail
        .ok_or_else(|| code_err(CodeError::FILE_UPLOAD_ERROR, "Missing thumbnail image"))?;
    let title = required_text(assets.title, "title")?;
    let description = required_text(assets.description, "description")?;

    let bundle_path = bundle.source.path().to_path_buf();
    let is_gzipped = bundle.is_gzipped;
    let is_html = bundle.is_html;
    let normalized = tokio::task::spawn_blocking(move || {
        normalize_bundle_file(
            &bundle_path,
            is_gzipped,
            is_html,
            MAX_BUNDLE_SIZE_BYTES as usize,
        )
    })
    .await
    .map_err(|source| code_err(CodeError::FILE_UPLOAD_ERROR, source))?
    .map_err(|source| code_err(CodeError::FILE_UPLOAD_ERROR, source))?;
    drop(bundle.source);

    let mut outputs = process_uploaded_image_files(
        thumbnail.path(),
        None,
        vec![CyhdevImageType::DemoThumbnail],
    )
    .await
    .map_err(|source| code_err(CodeError::COULD_NOT_PROCESS_IMAGE, source))?
    .into_iter();
    let processed_thumbnail = outputs.next().ok_or_else(|| {
        code_err(
            CodeError::COULD_NOT_PROCESS_IMAGE,
            "Thumbnail encoder produced no output",
        )
    })?;
    drop(thumbnail);

    let module_id = Uuid::now_v7();
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
    let thumbnail_url = location.public_s3_url(&region);
    let pending = [PendingMediaObject {
        location,
        content_type: "image/avif".to_string(),
        source: processed_thumbnail.path_buf(),
    }];
    let store = S3MediaObjectStore::from_config(&state.aws_profile_picture_config);
    let content_type = normalized.content_type;
    let persistence_state = Arc::clone(&state);
    let result = persist_media_objects(&store, &pending, async move {
        let mut connection = persistence_state
            .get_conn()
            .await
            .map_err(WasmPersistenceError::Pool)?;
        let now = Utc::now();
        let module: WasmModule = diesel::insert_into(wasm_module::table)
            .values(WasmModuleInsertable {
                wasm_module_id: module_id,
                user_id,
                wasm_module_link: format!("/api/wasm-modules/{module_id}/wasm"),
                wasm_module_description: description,
                wasm_module_created_at: now,
                wasm_module_updated_at: now,
                wasm_module_thumbnail_link: thumbnail_url,
                wasm_module_title: title,
                wasm_module_bundle_gz: normalized.gz_bytes,
            })
            .get_result(&mut connection)
            .await
            .map_err(WasmPersistenceError::Insert)?;
        drop(connection);
        Ok(PersistedMedia::new(module, Vec::new()))
    })
    .await;

    let module = match result {
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
            let code = match &source {
                WasmPersistenceError::Pool(_) => CodeError::POOL_ERROR,
                WasmPersistenceError::Insert(_) => CodeError::DB_INSERTION_ERROR,
            };
            return Err(code_err(code, source));
        }
    };
    state
        .upsert_wasm_module_cache(
            module_id,
            module.wasm_module_bundle_gz.clone(),
            content_type,
        )
        .await;
    info!(wasm_module_id = %module_id, user_id = %user_id, "WASM module uploaded");
    Ok(http_resp(WasmModuleItem::from(module), (), start))
}

fn required_text(value: Option<String>, name: &'static str) -> HandlerResponse<String> {
    value
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| code_err(CodeError::FILE_UPLOAD_ERROR, format!("Missing {name} field")))
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
