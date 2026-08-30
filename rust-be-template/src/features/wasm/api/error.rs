//! HTTP mapping for WebAssembly use-case failures.

use crate::errors::code_error::{CodeError, CodeErrorResp, code_err};

use super::super::error::WasmError;

#[derive(Clone, Copy)]
pub enum WasmMutation {
    Query,
    Insert,
    Update,
    Delete,
}

pub fn map_wasm_error(error: WasmError, mutation: WasmMutation) -> CodeErrorResp {
    let code = match &error {
        WasmError::Pool(_) => CodeError::POOL_ERROR,
        WasmError::Unauthorized => CodeError::IS_NOT_SUPERUSER,
        WasmError::ServiceBusy => CodeError::WASM_SERVICE_BUSY,
        WasmError::NotFound => CodeError::WASM_MODULE_NOT_FOUND,
        WasmError::Bundle(_) => CodeError::WASM_INVALID_BUNDLE,
        WasmError::Task(_) | WasmError::ObjectStore(_) => CodeError::FILE_UPLOAD_ERROR,
        WasmError::Image(_) => CodeError::COULD_NOT_PROCESS_IMAGE,
        WasmError::Database(_) => match mutation {
            WasmMutation::Query => CodeError::DB_QUERY_ERROR,
            WasmMutation::Insert => CodeError::DB_INSERTION_ERROR,
            WasmMutation::Update => CodeError::DB_UPDATE_ERROR,
            WasmMutation::Delete => CodeError::DB_DELETION_ERROR,
        },
    };
    code_err(code, error)
}
