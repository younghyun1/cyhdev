//! Bounded JSON extraction for blog write endpoints.

use axum::{
    body::to_bytes,
    extract::{FromRequest, Request},
    http::{StatusCode, header::CONTENT_TYPE},
};
use serde::de::DeserializeOwned;
use tracing::Level;

use crate::errors::code_error::{CodeError, CodeErrorResp, code_err};

pub const BLOG_JSON_BODY_MAX_BYTES: usize = 1024 * 1024;

const BLOG_PAYLOAD_TOO_LARGE: CodeError = CodeError {
    success: false,
    error_code: 45,
    http_status_code: StatusCode::PAYLOAD_TOO_LARGE,
    message: "Blog request body is too large!",
    log_level: Level::INFO,
};

pub struct BlogJson<T>(pub T);

impl<S, T> FromRequest<S> for BlogJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Send,
{
    type Rejection = CodeErrorResp;

    async fn from_request(request: Request, _state: &S) -> Result<Self, Self::Rejection> {
        if !is_json_content_type(request.headers().get(CONTENT_TYPE)) {
            return Err(code_err(
                CodeError::INVALID_REQUEST,
                "Content-Type must be application/json",
            ));
        }
        let bytes = to_bytes(request.into_body(), BLOG_JSON_BODY_MAX_BYTES)
            .await
            .map_err(|error| code_err(BLOG_PAYLOAD_TOO_LARGE, error))?;
        serde_json::from_slice(&bytes)
            .map(Self)
            .map_err(|error| code_err(CodeError::INVALID_REQUEST, error))
    }
}

fn is_json_content_type(value: Option<&axum::http::HeaderValue>) -> bool {
    let Some(value) = value.and_then(|value| value.to_str().ok()) else {
        return false;
    };
    let media_type = value.split(';').next().map(str::trim).unwrap_or_default();
    media_type == "application/json" || media_type.ends_with("+json")
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderValue;

    use super::is_json_content_type;

    #[test]
    fn json_content_type_accepts_parameters_and_suffixes() {
        assert!(is_json_content_type(Some(&HeaderValue::from_static(
            "application/json; charset=utf-8"
        ))));
        assert!(is_json_content_type(Some(&HeaderValue::from_static(
            "application/problem+json"
        ))));
        assert!(!is_json_content_type(Some(&HeaderValue::from_static(
            "text/plain"
        ))));
    }
}
