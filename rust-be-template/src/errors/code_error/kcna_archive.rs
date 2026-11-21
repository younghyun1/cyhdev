//! Stable client-visible errors for the KCNA research archive.
use super::CodeError;
use axum::http::StatusCode;
use tracing::Level;

impl CodeError {
    pub const KCNA_ARCHIVE_DOCUMENT_NOT_FOUND: Self = Self {
        success: false,
        error_code: 112,
        http_status_code: StatusCode::NOT_FOUND,
        message: "Archive document was not found.",
        log_level: Level::INFO,
    };
    pub const KCNA_ARCHIVE_NOT_INSTALLED: Self = Self {
        success: false,
        error_code: 113,
        http_status_code: StatusCode::SERVICE_UNAVAILABLE,
        message: "The archive snapshot is not installed on this server.",
        log_level: Level::INFO,
    };
    pub const KCNA_ARCHIVE_LOADING: Self = Self {
        success: false,
        error_code: 114,
        http_status_code: StatusCode::SERVICE_UNAVAILABLE,
        message: "The archive snapshot is still loading. Try again shortly.",
        log_level: Level::INFO,
    };
    pub const KCNA_ARCHIVE_LOAD_FAILED: Self = Self {
        success: false,
        error_code: 115,
        http_status_code: StatusCode::SERVICE_UNAVAILABLE,
        message: "The archive snapshot failed to load. See the server logs.",
        log_level: Level::WARN,
    };
    pub const KCNA_ARCHIVE_RELOAD_BUSY: Self = Self {
        success: false,
        error_code: 116,
        http_status_code: StatusCode::CONFLICT,
        message: "An archive snapshot reload is already in progress.",
        log_level: Level::INFO,
    };
    pub const KCNA_ARCHIVE_RELOAD_FAILED: Self = Self {
        success: false,
        error_code: 117,
        http_status_code: StatusCode::INTERNAL_SERVER_ERROR,
        message: "The archive snapshot reload failed; the previous snapshot keeps serving.",
        log_level: Level::ERROR,
    };
    pub const KCNA_ARCHIVE_INTERNAL: Self = Self {
        success: false,
        error_code: 118,
        http_status_code: StatusCode::INTERNAL_SERVER_ERROR,
        message: "The archive query failed.",
        log_level: Level::ERROR,
    };
    pub const KCNA_ARCHIVE_ACCESS_DENIED: Self = Self {
        success: false,
        error_code: 119,
        http_status_code: StatusCode::FORBIDDEN,
        message: "The archive is not open to this account.",
        log_level: Level::INFO,
    };
    pub const KCNA_ARCHIVE_ACCESS_STORE_FAILED: Self = Self {
        success: false,
        error_code: 120,
        http_status_code: StatusCode::INTERNAL_SERVER_ERROR,
        message: "The archive access list could not be read or changed.",
        log_level: Level::ERROR,
    };
    pub const KCNA_ARCHIVE_READER_NOT_FOUND: Self = Self {
        success: false,
        error_code: 121,
        http_status_code: StatusCode::NOT_FOUND,
        message: "There is no active account to add to the archive access list.",
        log_level: Level::INFO,
    };
    pub const KCNA_ARCHIVE_READER_LIMIT: Self = Self {
        success: false,
        error_code: 122,
        http_status_code: StatusCode::CONFLICT,
        message: "The archive access list is full.",
        log_level: Level::INFO,
    };
}
