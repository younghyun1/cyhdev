use std::sync::Arc;

use axum::{
    Extension,
    extract::{Path, State},
    response::IntoResponse,
};
use serde_derive::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    dto::responses::response_data::http_resp,
    errors::code_error::{CodeErrorResp, HandlerResponse},
    init::state::ServerState,
    util::time::now::tokio_now,
};

use super::error::{WasmMutation, map_wasm_error};

#[derive(Debug, Serialize, ToSchema)]
pub struct DeleteWasmModuleResponse {
    pub deleted_wasm_module_id: Uuid,
    pub cleanup_deleted_count: usize,
    pub cleanup_failure_count: usize,
    pub cleanup_remaining_count: usize,
    pub unresolved_cleanup_count: usize,
}

#[utoipa::path(
    delete,
    path = "/api/wasm-modules/{wasm_module_id}",
    tag = "wasm_module",
    params(("wasm_module_id" = Uuid, Path, description = "WASM module UUID")),
    responses(
        (status = 200, description = "WASM module deleted", body = DeleteWasmModuleResponse),
        (status = 401, description = "Unauthorized", body = CodeErrorResp),
        (status = 403, description = "Forbidden", body = CodeErrorResp),
        (status = 404, description = "WASM module not found", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn delete_wasm_module(
    Extension(user_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    Path(module_id): Path<Uuid>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let outcome = state
        .wasm_service()
        .delete_module(user_id, module_id)
        .await
        .map_err(|error| map_wasm_error(error, WasmMutation::Delete))?;
    Ok(http_resp(
        DeleteWasmModuleResponse {
            deleted_wasm_module_id: outcome.module_id,
            cleanup_deleted_count: outcome.cleanup.deleted_count,
            cleanup_failure_count: outcome.cleanup.failure_count,
            cleanup_remaining_count: outcome.cleanup.remaining_count,
            unresolved_cleanup_count: outcome.cleanup.unresolved_count,
        },
        (),
        start,
    ))
}
