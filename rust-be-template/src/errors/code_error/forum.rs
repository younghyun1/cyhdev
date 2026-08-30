use axum::http::StatusCode;
use tracing::Level;

use super::CodeError;

impl CodeError {
    pub const FORUM_INVALID_REQUEST: CodeError = CodeError {
        success: false,
        error_code: 73,
        http_status_code: StatusCode::BAD_REQUEST,
        message: "The forum request is invalid.",
        log_level: Level::INFO,
    };
    pub const FORUM_TOPIC_NOT_FOUND: CodeError = CodeError {
        success: false,
        error_code: 74,
        http_status_code: StatusCode::NOT_FOUND,
        message: "The forum topic was not found.",
        log_level: Level::INFO,
    };
    pub const FORUM_REPLY_NOT_FOUND: CodeError = CodeError {
        success: false,
        error_code: 75,
        http_status_code: StatusCode::NOT_FOUND,
        message: "The forum reply was not found.",
        log_level: Level::INFO,
    };
    pub const FORUM_CONTENT_CONFLICT: CodeError = CodeError {
        success: false,
        error_code: 76,
        http_status_code: StatusCode::CONFLICT,
        message: "The forum content state conflicts with this operation.",
        log_level: Level::INFO,
    };
    pub const FORUM_REVISION_CONFLICT: CodeError = CodeError {
        success: false,
        error_code: 77,
        http_status_code: StatusCode::CONFLICT,
        message: "The forum content changed; refresh before retrying.",
        log_level: Level::INFO,
    };
    pub const FORUM_FORBIDDEN: CodeError = CodeError {
        success: false,
        error_code: 78,
        http_status_code: StatusCode::FORBIDDEN,
        message: "The current account cannot perform this forum operation.",
        log_level: Level::WARN,
    };
    pub const FORUM_SUBSCRIPTION_SATURATED: CodeError = CodeError {
        success: false,
        error_code: 79,
        http_status_code: StatusCode::CONFLICT,
        message: "The forum topic cannot accept more subscriptions.",
        log_level: Level::WARN,
    };
    pub const FORUM_NOTIFICATION_NOT_FOUND: CodeError = CodeError {
        success: false,
        error_code: 80,
        http_status_code: StatusCode::NOT_FOUND,
        message: "The forum notification was not found.",
        log_level: Level::INFO,
    };
    pub const FORUM_WRITE_THROTTLED: CodeError = CodeError {
        success: false,
        error_code: 81,
        http_status_code: StatusCode::TOO_MANY_REQUESTS,
        message: "The forum write budget is exhausted; retry later.",
        log_level: Level::WARN,
    };
}
