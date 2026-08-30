use axum::http::StatusCode;
use tracing::Level;

use super::CodeError;

impl CodeError {
    pub const POOL_ERROR: CodeError = CodeError {
        success: false,
        error_code: 0,
        http_status_code: StatusCode::INTERNAL_SERVER_ERROR,
        message: "Could not get conn out of pool!",
        log_level: Level::ERROR,
    };
    pub const DB_QUERY_ERROR: CodeError = CodeError {
        success: false,
        error_code: 1,
        http_status_code: StatusCode::INTERNAL_SERVER_ERROR,
        message: "Database query failed!",
        log_level: Level::ERROR,
    };
    pub const DB_INSERTION_ERROR: CodeError = CodeError {
        success: false,
        error_code: 5,
        http_status_code: StatusCode::INTERNAL_SERVER_ERROR,
        message: "Database insertion failed!",
        log_level: Level::ERROR,
    };
    pub const DB_UPDATE_ERROR: CodeError = CodeError {
        success: false,
        error_code: 7,
        http_status_code: StatusCode::INTERNAL_SERVER_ERROR,
        message: "Database update failed!",
        log_level: Level::ERROR,
    };
    pub const DB_DELETION_ERROR: CodeError = CodeError {
        success: false,
        error_code: 30,
        http_status_code: StatusCode::INTERNAL_SERVER_ERROR,
        message: "Database deletion failed!",
        log_level: Level::ERROR,
    };
    pub const COULD_NOT_CREATE_DIRECTORY: CodeError = CodeError {
        success: false,
        error_code: 35,
        http_status_code: StatusCode::INTERNAL_SERVER_ERROR,
        message: "Could not create directory!",
        log_level: Level::ERROR,
    };
    pub const COULD_NOT_WRITE_FILE: CodeError = CodeError {
        success: false,
        error_code: 36,
        http_status_code: StatusCode::INTERNAL_SERVER_ERROR,
        message: "Could not write file to disk!",
        log_level: Level::ERROR,
    };
    pub const JOIN_ERROR: CodeError = CodeError {
        success: false,
        error_code: 37,
        http_status_code: StatusCode::INTERNAL_SERVER_ERROR,
        message: "Failed to perform async task join operation!",
        log_level: Level::ERROR,
    };
    pub const COULD_NOT_GET_I18N_BUNDLE: CodeError = CodeError {
        success: false,
        error_code: 38,
        http_status_code: StatusCode::INTERNAL_SERVER_ERROR,
        message: "Could not get i18n bundle!",
        log_level: Level::ERROR,
    };
    pub const COULD_NOT_SYNC_18N_CACHE: CodeError = CodeError {
        success: false,
        error_code: 39,
        http_status_code: StatusCode::INTERNAL_SERVER_ERROR,
        message: "Could not sync i18n cache!",
        log_level: Level::ERROR,
    };
    pub const COULD_NOT_RUN_FASTFETCH: CodeError = CodeError {
        success: false,
        error_code: 40,
        http_status_code: StatusCode::INTERNAL_SERVER_ERROR,
        message: "Could not run fastfetch!",
        log_level: Level::ERROR,
    };
    pub const INVALID_REQUEST: CodeError = CodeError {
        success: false,
        error_code: 45,
        http_status_code: StatusCode::BAD_REQUEST,
        message: "Invalid request!",
        log_level: Level::INFO,
    };
}

