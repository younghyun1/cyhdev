use std::sync::Arc;

use axum::{extract::State, response::IntoResponse};

use crate::{
    dto::responses::{
        response_data::http_resp,
        wasm_module::{GetWasmModulesResponse, WasmModuleItem},
    },
    errors::code_error::{CodeErrorResp, HandlerResponse},
    init::state::ServerState,
    util::time::now::tokio_now,
};

use super::error::{WasmMutation, map_wasm_error};

/// Returns at most the newest 1,024 modules for compatibility with the existing shape.
#[utoipa::path(
    get,
    path = "/api/wasm-modules",
    tag = "wasm_module",
    responses(
        (status = 200, description = "List of WASM modules", body = GetWasmModulesResponse),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn get_wasm_modules(
    State(state): State<Arc<ServerState>>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let modules = state
        .wasm_service()
        .list_modules()
        .await
        .map_err(|error| map_wasm_error(error, WasmMutation::Query))?;
    let items = modules.into_iter().map(WasmModuleItem::from).collect();
    Ok(http_resp(GetWasmModulesResponse { items }, (), start))
}
