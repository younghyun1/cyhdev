use axum::http::StatusCode;
use tracing::Level;

use super::CodeError;

impl CodeError {
    pub const POST_TITLE_NOT_UNIQUE: CodeError = CodeError {
        success: false,
        error_code: 23,
        http_status_code: StatusCode::BAD_REQUEST,
        message: "Post title must be unique!",
        log_level: Level::INFO,
    };
    pub const POST_NOT_FOUND: CodeError = CodeError {
        success: false,
        error_code: 24,
        http_status_code: StatusCode::NOT_FOUND,
        message: "Post not found or access denied!",
        log_level: Level::INFO,
    };
    pub const COMMENT_NOT_FOUND: CodeError = CodeError {
        success: false,
        error_code: 46,
        http_status_code: StatusCode::NOT_FOUND,
        message: "Comment not found or access denied!",
        log_level: Level::INFO,
    };
    pub const UPVOTE_MUST_BE_UNIQUE: CodeError = CodeError {
        success: false,
        error_code: 29,
        http_status_code: StatusCode::BAD_REQUEST,
        message: "Upvote must be unique!",
        log_level: Level::INFO,
    };
    pub const UPVOTE_DOES_NOT_EXIST: CodeError = CodeError {
        success: false,
        error_code: 31,
        http_status_code: StatusCode::BAD_REQUEST,
        message: "Upvote does not exist - could not delete.",
        log_level: Level::INFO,
    };
    pub const FILE_UPLOAD_ERROR: CodeError = CodeError {
        success: false,
        error_code: 32,
        http_status_code: StatusCode::INTERNAL_SERVER_ERROR,
        message: "File upload failed!",
        log_level: Level::ERROR,
    };
    pub const IMAGE_TOO_LARGE: CodeError = CodeError {
        success: false,
        error_code: 33,
        http_status_code: StatusCode::BAD_REQUEST,
        message: "Image too large! Maximum allowed size is 10MB.",
        log_level: Level::INFO,
    };
    pub const COULD_NOT_PROCESS_IMAGE: CodeError = CodeError {
        success: false,
        error_code: 34,
        http_status_code: StatusCode::INTERNAL_SERVER_ERROR,
        message: "Could not process the uploaded image!",
        log_level: Level::ERROR,
    };
    pub const POST_NOT_FOUND_IN_CACHE: CodeError = CodeError {
        success: false,
        error_code: 43,
        http_status_code: StatusCode::NOT_FOUND,
        message: "Post not found in RAM cache!",
        log_level: Level::INFO,
    };
    pub const POST_CACHE_INSERTION_ERROR: CodeError = CodeError {
        success: false,
        error_code: 44,
        http_status_code: StatusCode::INTERNAL_SERVER_ERROR,
        message: "Post cache insertion failed!",
        log_level: Level::ERROR,
    };
    pub const BATCH_NOT_FOUND: CodeError = CodeError {
        success: false,
        error_code: 47,
        http_status_code: StatusCode::NOT_FOUND,
        message: "Processing batch not found!",
        log_level: Level::INFO,
    };
    pub const BATCH_TOO_MANY_FILES: CodeError = CodeError {
        success: false,
        error_code: 48,
        http_status_code: StatusCode::BAD_REQUEST,
        message: "Too many files in batch!",
        log_level: Level::WARN,
    };
    pub const BATCH_EMPTY: CodeError = CodeError {
        success: false,
        error_code: 49,
        http_status_code: StatusCode::BAD_REQUEST,
        message: "Batch contains no files!",
        log_level: Level::INFO,
    };
    pub const PHOTOGRAPH_NOT_FOUND: CodeError = CodeError {
        success: false,
        error_code: 50,
        http_status_code: StatusCode::NOT_FOUND,
        message: "Photograph not found!",
        log_level: Level::INFO,
    };
}
