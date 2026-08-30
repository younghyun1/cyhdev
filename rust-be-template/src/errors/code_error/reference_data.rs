use axum::http::StatusCode;
use tracing::Level;

use super::CodeError;

impl CodeError {
    pub const INVALID_IP_ADDRESS: CodeError = CodeError {
        success: false,
        error_code: 25,
        http_status_code: StatusCode::BAD_REQUEST,
        message: "Invalid IP address provided!",
        log_level: Level::INFO,
    };
    pub const LANGUAGE_NOT_FOUND: CodeError = CodeError {
        success: false,
        error_code: 27,
        http_status_code: StatusCode::NOT_FOUND,
        message: "Language not found!",
        log_level: Level::INFO,
    };
    pub const COUNTRY_NOT_FOUND: CodeError = CodeError {
        success: false,
        error_code: 28,
        http_status_code: StatusCode::NOT_FOUND,
        message: "Country not found!",
        log_level: Level::INFO,
    };
}

