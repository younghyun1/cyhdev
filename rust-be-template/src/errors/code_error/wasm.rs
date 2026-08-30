use axum::http::StatusCode;
use tracing::Level;

use super::CodeError;

impl CodeError {
    pub const WASM_MODULE_NOT_FOUND: CodeError = CodeError {
        success: false,
        error_code: 82,
        http_status_code: StatusCode::NOT_FOUND,
        message: "The WebAssembly module was not found.",
        log_level: Level::INFO,
    };
    pub const WASM_INVALID_BUNDLE: CodeError = CodeError {
        success: false,
        error_code: 83,
        http_status_code: StatusCode::BAD_REQUEST,
        message: "The WebAssembly bundle is invalid.",
        log_level: Level::INFO,
    };
    pub const WASM_SERVICE_BUSY: CodeError = CodeError {
        success: false,
        error_code: 84,
        http_status_code: StatusCode::SERVICE_UNAVAILABLE,
        message: "WebAssembly processing capacity is temporarily unavailable.",
        log_level: Level::WARN,
    };
    pub const UPLOAD_TOO_LARGE: CodeError = CodeError {
        success: false,
        error_code: 85,
        http_status_code: StatusCode::PAYLOAD_TOO_LARGE,
        message: "The upload exceeds its configured size limit.",
        log_level: Level::INFO,
    };
}
