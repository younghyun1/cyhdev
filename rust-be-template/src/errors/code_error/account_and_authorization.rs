use axum::http::StatusCode;
use tracing::Level;

use super::CodeError;

impl CodeError {
    pub const SYSTEM_ACTOR_PROTECTED: CodeError = CodeError {
        success: false,
        error_code: 53,
        http_status_code: StatusCode::FORBIDDEN,
        message: "The protected system actor cannot be modified!",
        log_level: Level::WARN,
    };
    pub const ACCOUNT_LIFECYCLE_CONFLICT: CodeError = CodeError {
        success: false,
        error_code: 54,
        http_status_code: StatusCode::CONFLICT,
        message: "The account lifecycle state conflicts with this operation!",
        log_level: Level::INFO,
    };
    pub const ACCOUNT_PASSWORD_CONFIRMATION_FAILED: CodeError = CodeError {
        success: false,
        error_code: 55,
        http_status_code: StatusCode::BAD_REQUEST,
        message: "The current password confirmation did not match!",
        log_level: Level::INFO,
    };
    pub const MEDIA_CLEANUP_NOT_FOUND: CodeError = CodeError {
        success: false,
        error_code: 56,
        http_status_code: StatusCode::NOT_FOUND,
        message: "Media cleanup record was not found!",
        log_level: Level::INFO,
    };
    pub const MEDIA_CLEANUP_CONFLICT: CodeError = CodeError {
        success: false,
        error_code: 57,
        http_status_code: StatusCode::CONFLICT,
        message: "Media cleanup reconciliation conflicts with stored state!",
        log_level: Level::INFO,
    };
    pub const PROFILE_PICTURE_NOT_FOUND: CodeError = CodeError {
        success: false,
        error_code: 58,
        http_status_code: StatusCode::NOT_FOUND,
        message: "Profile picture was not found!",
        log_level: Level::INFO,
    };
    pub const AUTHORIZATION_CONFLICT: CodeError = CodeError {
        success: false,
        error_code: 70,
        http_status_code: StatusCode::CONFLICT,
        message: "The requested authorization change conflicts with current authority.",
        log_level: Level::WARN,
    };
    pub const AUTHORIZATION_RESOURCE_NOT_FOUND: CodeError = CodeError {
        success: false,
        error_code: 71,
        http_status_code: StatusCode::NOT_FOUND,
        message: "The requested authorization resource was not found.",
        log_level: Level::INFO,
    };
    pub const AUTHORIZATION_DATA_INTEGRITY: CodeError = CodeError {
        success: false,
        error_code: 72,
        http_status_code: StatusCode::INTERNAL_SERVER_ERROR,
        message: "Stored authorization data failed validation.",
        log_level: Level::ERROR,
    };
    pub const IS_NOT_SUPERUSER: CodeError = CodeError {
        success: false,
        error_code: 42,
        http_status_code: StatusCode::FORBIDDEN,
        message: "Operation requires superuser privileges!",
        log_level: Level::WARN,
    };
}
