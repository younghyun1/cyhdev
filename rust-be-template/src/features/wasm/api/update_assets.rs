use std::sync::Arc;

use axum::{
    Extension,
    extract::{Multipart, Path, State},
};
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

use super::{
    asset_upload::read_assets,
    error::{WasmMutation, map_wasm_error},
};

#[utoipa::path(
    post,
    path = "/api/wasm-modules/{wasm_module_id}/assets",
    tag = "wasm_module",
    params(("wasm_module_id" = Uuid, Path, description = "WASM module UUID")),
    request_body(content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "WASM module updated", body = WasmModuleItem),
        (status = 400, description = "Invalid upload payload", body = CodeErrorResp),
        (status = 413, description = "Upload payload is too large", body = CodeErrorResp),
        (status = 401, description = "Unauthorized", body = CodeErrorResp),
        (status = 403, description = "Forbidden", body = CodeErrorResp),
        (status = 404, description = "WASM module not found", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn update_wasm_module_assets(
    Extension(user_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    Path(module_id): Path<Uuid>,
    mut multipart: Multipart,
) -> HandlerResponse<ApiResponse<WasmModuleItem, ()>> {
    let start = tokio_now();
    let assets = read_assets(&mut multipart).await?;
    let module = state
        .wasm_service()
        .update_assets(user_id, module_id, assets)
        .await
        .map_err(|error| map_wasm_error(error, WasmMutation::Update))?;
    Ok(http_resp(WasmModuleItem::from(module), (), start))
}
