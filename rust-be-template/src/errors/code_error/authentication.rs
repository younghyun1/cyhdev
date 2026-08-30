use axum::http::StatusCode;
use tracing::Level;

use super::CodeError;

impl CodeError {
    pub const EMAIL_INVALID: CodeError = CodeError {
        success: false,
        error_code: 2,
        http_status_code: StatusCode::BAD_REQUEST,
        message: "Invalid email address!",
        log_level: Level::INFO, // info, debug, trace all info'd
    };
    pub const USER_NAME_INVALID: CodeError = CodeError {
        success: false,
        error_code: 3,
        http_status_code: StatusCode::BAD_REQUEST,
        message: "Invalid username!",
        log_level: Level::INFO,
    };
    pub const COULD_NOT_HASH_PW: CodeError = CodeError {
        success: false,
        error_code: 4,
        http_status_code: StatusCode::INTERNAL_SERVER_ERROR,
        message: "Failed to hash the password!",
        log_level: Level::ERROR,
    };
    pub const EMAIL_MUST_BE_UNIQUE: CodeError = CodeError {
        success: false,
        error_code: 6,
        http_status_code: StatusCode::BAD_REQUEST,
        message: "Email address already exists!",
        log_level: Level::INFO,
    };
    pub const INVALID_EMAIL_VERIFICATION_TOKEN: CodeError = CodeError {
        success: false,
        error_code: 8,
        http_status_code: StatusCode::BAD_REQUEST,
        message: "Invalid email verification token!",
        log_level: Level::INFO,
    };
    pub const EMAIL_VERIFICATION_TOKEN_EXPIRED: CodeError = CodeError {
        success: false,
        error_code: 9,
        http_status_code: StatusCode::BAD_REQUEST,
        message: "Email verification token has expired!",
        log_level: Level::INFO,
    };
    pub const EMAIL_VERIFICATION_TOKEN_FABRICATED: CodeError = CodeError {
        success: false,
        error_code: 10,
        http_status_code: StatusCode::BAD_REQUEST,
        message: "Email verification token was fabricated; created_at was in the future!",
        log_level: Level::ERROR,
    };
    pub const EMAIL_VERIFICATION_TOKEN_ALREADY_USED: CodeError = CodeError {
        success: false,
        error_code: 11,
        http_status_code: StatusCode::BAD_REQUEST,
        message: "Email verification token has already been used!",
        log_level: Level::INFO,
    };
    pub const USER_EMAIL_ALREADY_VERIFIED: CodeError = CodeError {
        success: false,
        error_code: 12,
        http_status_code: StatusCode::BAD_REQUEST,
        message: "User email is already verified!",
        log_level: Level::INFO,
    };
    pub const PASSWORD_INVALID: CodeError = CodeError {
        success: false,
        error_code: 13,
        http_status_code: StatusCode::BAD_REQUEST,
        message: "Invalid password form! Must contain lower and uppercase characters and digits.",
        log_level: Level::INFO,
    };
    pub const USER_NOT_FOUND: CodeError = CodeError {
        success: false,
        error_code: 14,
        http_status_code: StatusCode::NOT_FOUND,
        message: "User not found!",
        log_level: Level::INFO,
    };
    pub const WRONG_PW: CodeError = CodeError {
        success: false,
        error_code: 15,
        http_status_code: StatusCode::UNAUTHORIZED,
        message: "Incorrect password!",
        log_level: Level::INFO,
    };
    pub const COULD_NOT_VERIFY_PW: CodeError = CodeError {
        success: false,
        error_code: 16,
        http_status_code: StatusCode::INTERNAL_SERVER_ERROR,
        message: "Wrong password!",
        log_level: Level::INFO,
    };
    pub const PASSWORD_RESET_TOKEN_FABRICATED: CodeError = CodeError {
        success: false,
        error_code: 19,
        http_status_code: StatusCode::BAD_REQUEST,
        message: "Password reset token was fabricated; created_at was in the future!",
        log_level: Level::ERROR,
    };
    pub const PASSWORD_RESET_TOKEN_ALREADY_USED: CodeError = CodeError {
        success: false,
        error_code: 20,
        http_status_code: StatusCode::BAD_REQUEST,
        message: "Password reset token has already been used!",
        log_level: Level::INFO,
    };
    pub const PASSWORD_RESET_TOKEN_EXPIRED: CodeError = CodeError {
        success: false,
        error_code: 21,
        http_status_code: StatusCode::BAD_REQUEST,
        message: "Password reset token has expired!",
        log_level: Level::INFO,
    };
    pub const UNAUTHORIZED_ACCESS: CodeError = CodeError {
        success: false,
        error_code: 22,
        http_status_code: StatusCode::UNAUTHORIZED,
        message: "Unauthorized access attempt!",
        log_level: Level::WARN,
    };
    pub const UNTRUSTED_ORIGIN: CodeError = CodeError {
        success: false,
        error_code: 26,
        http_status_code: StatusCode::FORBIDDEN,
        message: "Request origin is not trusted!",
        log_level: Level::WARN,
    };
    pub const SESSION_CREATION_FAILED: CodeError = CodeError {
        success: false,
        error_code: 51,
        http_status_code: StatusCode::INTERNAL_SERVER_ERROR,
        message: "Could not create a session!",
        log_level: Level::ERROR,
    };
    pub const SESSION_STORE_SATURATED: CodeError = CodeError {
        success: false,
        error_code: 52,
        http_status_code: StatusCode::SERVICE_UNAVAILABLE,
        message: "Session capacity is temporarily unavailable!",
        log_level: Level::WARN,
    };
    pub const AUTH_THROTTLED: CodeError = CodeError {
        success: false,
        error_code: 59,
        http_status_code: StatusCode::TOO_MANY_REQUESTS,
        message: "Too many authentication attempts. Try again later.",
        log_level: Level::WARN,
    };
    pub const INVALID_CREDENTIALS: CodeError = CodeError {
        success: false,
        error_code: 60,
        http_status_code: StatusCode::UNAUTHORIZED,
        message: "Invalid email or password.",
        log_level: Level::INFO,
    };
    pub const ACCOUNT_IDENTITY_UNAVAILABLE: CodeError = CodeError {
        success: false,
        error_code: 61,
        http_status_code: StatusCode::CONFLICT,
        message: "The requested account identity is unavailable.",
        log_level: Level::INFO,
    };
    pub const PASSWORD_RESET_REJECTED: CodeError = CodeError {
        success: false,
        error_code: 62,
        http_status_code: StatusCode::BAD_REQUEST,
        message: "The password reset request is invalid or no longer active.",
        log_level: Level::INFO,
    };
    pub const OIDC_DISABLED: CodeError = CodeError {
        success: false,
        error_code: 63,
        http_status_code: StatusCode::NOT_FOUND,
        message: "OpenID Connect is not configured.",
        log_level: Level::INFO,
    };
    pub const OIDC_FLOW_REJECTED: CodeError = CodeError {
        success: false,
        error_code: 64,
        http_status_code: StatusCode::BAD_REQUEST,
        message: "The OpenID Connect flow is invalid or no longer active.",
        log_level: Level::WARN,
    };
    pub const OIDC_IDENTITY_NOT_LINKED: CodeError = CodeError {
        success: false,
        error_code: 65,
        http_status_code: StatusCode::UNAUTHORIZED,
        message: "The OpenID Connect identity is not linked to an active account.",
        log_level: Level::INFO,
    };
    pub const OIDC_IDENTITY_CONFLICT: CodeError = CodeError {
        success: false,
        error_code: 66,
        http_status_code: StatusCode::CONFLICT,
        message: "The OpenID Connect identity conflicts with existing account access.",
        log_level: Level::INFO,
    };
    pub const OIDC_TEMPORARILY_UNAVAILABLE: CodeError = CodeError {
        success: false,
        error_code: 67,
        http_status_code: StatusCode::SERVICE_UNAVAILABLE,
        message: "OpenID Connect is temporarily unavailable.",
        log_level: Level::WARN,
    };
    pub const EMAIL_NOT_VERIFIED: CodeError = CodeError {
        success: false,
        error_code: 41,
        http_status_code: StatusCode::BAD_REQUEST,
        message: "Email address is not verified!",
        log_level: Level::INFO,
    };
}

