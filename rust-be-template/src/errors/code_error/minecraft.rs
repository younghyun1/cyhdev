//! Stable client-visible errors for Minecraft management.
use super::CodeError;
use axum::http::StatusCode;
use tracing::Level;

impl CodeError {
    pub const MINECRAFT_DISABLED: Self = Self {
        success: false,
        error_code: 86,
        http_status_code: StatusCode::SERVICE_UNAVAILABLE,
        message: "Minecraft controls are not configured.",
        log_level: Level::INFO,
    };
    pub const MINECRAFT_BUSY: Self = Self {
        success: false,
        error_code: 87,
        http_status_code: StatusCode::TOO_MANY_REQUESTS,
        message: "A Minecraft operation is in progress or cooling down. Wait before retrying.",
        log_level: Level::INFO,
    };
    pub const MINECRAFT_UNCERTAIN: Self = Self {
        success: false,
        error_code: 88,
        http_status_code: StatusCode::SERVICE_UNAVAILABLE,
        message: "Minecraft did not acknowledge the request. Check its state before retrying.",
        log_level: Level::WARN,
    };
}
