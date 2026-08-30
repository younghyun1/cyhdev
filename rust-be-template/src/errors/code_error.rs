use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_derive::Serialize;
use std::error::Error;
use std::fmt::{self, Debug};
use tracing::Level;
use utoipa::ToSchema;

mod account_and_authorization;
mod authentication;
mod content_and_media;
mod forum;
mod reference_data;
mod runtime_and_request;
mod wasm;

pub type HandlerResponse<T> = Result<T, CodeErrorResp>;

#[derive(Copy, Clone, Debug)]
pub struct CodeError {
    pub success: bool,
    pub error_code: u8,
    pub http_status_code: StatusCode,
    pub message: &'static str,
    pub log_level: Level,
}

pub fn code_err(cerr: CodeError, e: impl ToString) -> CodeErrorResp {
    CodeErrorResp {
        success: cerr.success,
        error_code: cerr.error_code,
        http_status_code: cerr.http_status_code,
        message: cerr.message.to_string(),
        error_message: e.to_string(),
        log_level: cerr.log_level,
        retry_after_seconds: None,
    }
}

#[derive(Debug, Clone)]
pub struct CodeErrorLogContext {
    pub log_level: Level,
    pub status_code: StatusCode,
    pub error_code: u8,
    pub message: String,
    pub detail: String,
}

#[derive(Serialize, Debug, Clone, ToSchema)]
pub struct CodeErrorResp {
    pub success: bool,
    pub error_code: u8,
    #[serde(skip_serializing)]
    pub http_status_code: StatusCode,
    pub message: String,
    #[serde(skip_serializing)]
    pub error_message: String,
    #[serde(skip_serializing)]
    pub log_level: Level,
    #[serde(skip_serializing)]
    retry_after_seconds: Option<u64>,
}

impl CodeErrorResp {
    pub fn with_retry_after(mut self, retry_after: std::time::Duration) -> Self {
        let rounded = retry_after
            .as_secs()
            .saturating_add(u64::from(retry_after.subsec_nanos() > 0))
            .max(1);
        self.retry_after_seconds = Some(rounded);
        self
    }
}

// Implement std::fmt::Display for CodeErrorResp
impl fmt::Display for CodeErrorResp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.message, self.error_message)
    }
}

// Implement std::error::Error for CodeErrorResp
impl Error for CodeErrorResp {}

// Implement IntoResponse for CodeErrorResp
impl IntoResponse for CodeErrorResp {
    fn into_response(self) -> axum::response::Response {
        let body = Json(&self);
        let mut response = (self.http_status_code, body).into_response();

        if let Some(seconds) = self.retry_after_seconds {
            match axum::http::HeaderValue::from_str(&seconds.to_string()) {
                Ok(value) => {
                    response
                        .headers_mut()
                        .insert(axum::http::header::RETRY_AFTER, value);
                }
                Err(error) => {
                    tracing::warn!(error = %error, "Failed to encode Retry-After header");
                }
            }
        }

        response.extensions_mut().insert(CodeErrorLogContext {
            log_level: self.log_level,
            status_code: self.http_status_code,
            error_code: self.error_code,
            message: self.message.clone(),
            detail: self.error_message.clone(),
        });

        response
    }
}

// Implement From<CodeError> for CodeErrorResp
impl From<CodeError> for CodeErrorResp {
    fn from(cerr: CodeError) -> Self {
        CodeErrorResp {
            success: cerr.success,
            error_code: cerr.error_code,
            http_status_code: cerr.http_status_code,
            message: cerr.message.to_string(),
            error_message: "".to_string(),
            log_level: cerr.log_level,
            retry_after_seconds: None,
        }
    }
}

// Implement IntoResponse for CodeError
impl IntoResponse for CodeError {
    fn into_response(self) -> axum::response::Response {
        let resp: CodeErrorResp = self.into();
        resp.into_response()
    }
}
