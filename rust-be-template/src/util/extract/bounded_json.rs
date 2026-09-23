//! JSON extraction with a per-route byte ceiling.
//!
//! The router's default body limit is sized for media uploads. Small JSON write
//! endpoints use this extractor so a client cannot make the server buffer a
//! body far larger than any valid request before deserialization rejects it.
//! The body is read with its own limit, so the outer default limit never
//! applies to these routes.

use axum::{
    body::to_bytes,
    extract::{FromRequest, Request},
    http::{HeaderValue, StatusCode, header::CONTENT_TYPE},
};
use serde::de::DeserializeOwned;
use tracing::Level;

use crate::errors::code_error::{CodeError, CodeErrorResp, code_err};

/// Byte ceiling for comment and vote bodies. A 4,000-character comment is at
/// most 16,000 bytes of UTF-8; the extra room admits clients that escape control
/// characters or quotes as `\uXXXX`, so a domain-valid body is never rejected.
pub const SOCIAL_JSON_BODY_MAX_BYTES: usize = 32 * 1024;

const PAYLOAD_TOO_LARGE: CodeError = CodeError {
    success: false,
    error_code: 45,
    http_status_code: StatusCode::PAYLOAD_TOO_LARGE,
    message: "Request body is too large!",
    log_level: Level::INFO,
};

/// Deserializes a JSON body of at most `MAX_BYTES` bytes.
pub struct BoundedJson<T, const MAX_BYTES: usize>(pub T);

impl<S, T, const MAX_BYTES: usize> FromRequest<S> for BoundedJson<T, MAX_BYTES>
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
        let bytes = to_bytes(request.into_body(), MAX_BYTES)
            .await
            .map_err(|error| code_err(PAYLOAD_TOO_LARGE, error))?;
        serde_json::from_slice(&bytes)
            .map(Self)
            .map_err(|error| code_err(CodeError::INVALID_REQUEST, error))
    }
}

fn is_json_content_type(value: Option<&HeaderValue>) -> bool {
    let Some(value) = value.and_then(|value| value.to_str().ok()) else {
        return false;
    };
    let media_type = value.split(';').next().map(str::trim).unwrap_or_default();
    media_type == "application/json" || media_type.ends_with("+json")
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        extract::FromRequest,
        http::{HeaderValue, Request, StatusCode, header::CONTENT_TYPE},
    };
    use serde_derive::Deserialize;

    use super::{BoundedJson, is_json_content_type};

    #[derive(Deserialize)]
    struct Probe {
        value: String,
    }

    fn json_request(body: String) -> Request<Body> {
        let mut request = Request::new(Body::from(body));
        request
            .headers_mut()
            .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        request
    }

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

    #[tokio::test]
    async fn body_at_the_limit_is_accepted_and_one_byte_more_is_rejected() {
        let envelope = r#"{"value":""}"#.len();
        let fits = format!(r#"{{"value":"{}"}}"#, "x".repeat(64 - envelope));
        let accepted = BoundedJson::<Probe, 64>::from_request(json_request(fits), &()).await;
        assert!(matches!(accepted, Ok(BoundedJson(probe)) if probe.value.len() == 64 - envelope));

        let overflow = format!(r#"{{"value":"{}"}}"#, "x".repeat(65 - envelope));
        let rejected = BoundedJson::<Probe, 64>::from_request(json_request(overflow), &()).await;
        assert!(matches!(
            rejected,
            Err(error) if error.http_status_code == StatusCode::PAYLOAD_TOO_LARGE
        ));
    }
}
