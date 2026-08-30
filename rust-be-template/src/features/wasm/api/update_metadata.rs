use std::sync::Arc;

use axum::{Extension, Json, extract::{Path, State}, response::IntoResponse};
use uuid::Uuid;

use crate::{
    dto::{
        requests::wasm_module::UpdateWasmModuleRequest,
        responses::{response_data::http_resp, wasm_module::WasmModuleItem},
    },
    errors::code_error::{CodeErrorResp, HandlerResponse},
    init::state::ServerState,
    util::time::now::tokio_now,
};

use super::error::{WasmMutation, map_wasm_error};

#[utoipa::path(
    patch,
    path = "/api/wasm-modules/{wasm_module_id}",
    tag = "wasm_module",
    params(("wasm_module_id" = Uuid, Path, description = "WASM module UUID")),
    request_body = UpdateWasmModuleRequest,
    responses(
        (status = 200, description = "WASM module updated", body = WasmModuleItem),
        (status = 401, description = "Unauthorized", body = CodeErrorResp),
        (status = 403, description = "Forbidden", body = CodeErrorResp),
        (status = 404, description = "WASM module not found", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn update_wasm_module(
    Extension(user_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    Path(module_id): Path<Uuid>,
    Json(request): Json<UpdateWasmModuleRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let updated = state
        .wasm_service()
        .update_metadata(
            user_id,
            module_id,
            request.wasm_module_title,
            request.wasm_module_description,
        )
        .await
        .map_err(|error| map_wasm_error(error, WasmMutation::Update))?;
    Ok(http_resp(WasmModuleItem::from(updated), (), start))
}
