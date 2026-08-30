use std::sync::Arc;

use axum::{Extension, extract::{Multipart, State}};
use uuid::Uuid;

use crate::{
    dto::responses::{
        response_data::{Response as ApiResponse, http_resp},
        wasm_module::WasmModuleItem,
    },
    errors::code_error::{CodeErrorResp, HandlerResponse},
    init::state::ServerState,
    util::time::now::tokio_now,
};

use super::{asset_upload::read_assets, error::{WasmMutation, map_wasm_error}};

#[utoipa::path(
    post,
    path = "/api/wasm-modules",
    tag = "wasm_module",
    request_body(content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "WASM module uploaded successfully", body = WasmModuleItem),
        (status = 400, description = "Invalid upload payload", body = CodeErrorResp),
        (status = 413, description = "Upload payload is too large", body = CodeErrorResp),
        (status = 401, description = "Unauthorized", body = CodeErrorResp),
        (status = 403, description = "Forbidden", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn upload_wasm_module(
    Extension(user_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    mut multipart: Multipart,
) -> HandlerResponse<ApiResponse<WasmModuleItem, ()>> {
    let start = tokio_now();
    let assets = read_assets(&mut multipart).await?;
    let module = state
        .wasm_service()
        .create_module(user_id, assets)
        .await
        .map_err(|error| map_wasm_error(error, WasmMutation::Insert))?;
    Ok(http_resp(WasmModuleItem::from(module), (), start))
}
