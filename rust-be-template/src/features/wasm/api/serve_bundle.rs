use std::sync::Arc;

use axum::{
    body::{Body, Bytes},
    extract::{Path, State},
    http::{HeaderMap, Response, StatusCode, header},
    response::IntoResponse,
};
use tracing::{error, info};
use uuid::Uuid;

use crate::init::state::ServerState;
use crate::errors::code_error::{CodeError, CodeErrorResp};

#[utoipa::path(
    get,
    path = "/api/wasm-modules/{wasm_module_id}/wasm",
    tag = "wasm_module",
    params(("wasm_module_id" = Uuid, Path, description = "WASM module UUID")),
    responses(
        (status = 200, description = "WASM bundle", content_type = "application/wasm"),
        (status = 404, description = "WASM module not found"),
        (status = 503, description = "Identity decompression capacity is unavailable", body = CodeErrorResp)
    )
)]
pub async fn serve_wasm(
    State(state): State<Arc<ServerState>>,
    Path(module_id): Path<Uuid>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let accepts_gzip = accepts_gzip(&headers);
    let bundle = match state
        .wasm_service()
        .served_bundle(module_id, accepts_gzip)
        .await
    {
        Ok(Some(bundle)) => bundle,
        Ok(None) => return text_response(StatusCode::NOT_FOUND, "WASM module not found"),
        Err(crate::features::wasm::error::WasmError::ServiceBusy) => {
            return CodeErrorResp::from(CodeError::WASM_SERVICE_BUSY)
                .with_retry_after(std::time::Duration::from_secs(1))
                .into_response();
        }
        Err(error_value) => {
            error!(error = %error_value, wasm_module_id = %module_id, "Failed to serve WebAssembly bundle");
            return text_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to decode WASM bundle",
            );
        }
    };
    info!(
        wasm_module_id = %module_id,
        size_bytes = bundle.bytes.len(),
        is_gzipped = bundle.content_encoding_gzip,
        content_type = bundle.content_type,
        "Serving WebAssembly module bundle"
    );
    let body = Body::from(Bytes::from_owner(bundle.bytes));
    let mut response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, bundle.content_type)
        // Module IDs are stable while asset replacement mutates the bundle bytes.
        .header(header::CACHE_CONTROL, "public, max-age=0, must-revalidate")
        .header(header::VARY, header::ACCEPT_ENCODING.as_str())
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*");
    if bundle.content_encoding_gzip {
        response = response.header(header::CONTENT_ENCODING, "gzip");
    }
    if bundle.content_type.starts_with("text/html") {
        // HTML demos execute in an opaque sandboxed origin even though their
        // stable URL is hosted by the application server.
        response = response.header(
            header::CONTENT_SECURITY_POLICY,
            "sandbox allow-scripts",
        );
    }
    response = response.header(header::X_CONTENT_TYPE_OPTIONS, "nosniff");
    match response.body(body) {
        Ok(response) => response,
        Err(error_value) => {
            error!(error = %error_value, wasm_module_id = %module_id, "Failed to build WebAssembly response");
            text_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to build WASM response")
        }
    }
}

fn accepts_gzip(headers: &HeaderMap) -> bool {
    let mut explicit_gzip = None;
    let mut wildcard = None;
    for value in headers.get_all(header::ACCEPT_ENCODING) {
        let Ok(value) = value.to_str() else { continue };
        for encoding in value.split(',') {
            let mut parts = encoding.trim().split(';');
            let name = parts.next().unwrap_or_default().trim();
            let quality = encoding_quality(parts);
            if name.eq_ignore_ascii_case("gzip") || name.eq_ignore_ascii_case("x-gzip") {
                explicit_gzip = Some(explicit_gzip.unwrap_or(0.0_f32).max(quality));
            } else if name == "*" {
                wildcard = Some(wildcard.unwrap_or(0.0_f32).max(quality));
            }
        }
    }
    explicit_gzip.or(wildcard).is_some_and(|quality| quality > 0.0)
}

fn encoding_quality<'a>(parameters: impl Iterator<Item = &'a str>) -> f32 {
    for parameter in parameters {
        let Some((name, value)) = parameter.trim().split_once('=') else { continue };
        if !name.trim().eq_ignore_ascii_case("q") {
            continue;
        }
        return value
            .trim()
            .parse::<f32>()
            .ok()
            .filter(|quality| quality.is_finite() && (0.0..=1.0).contains(quality))
            .unwrap_or(0.0);
    }
    1.0
}

fn text_response(status: StatusCode, body: &'static str) -> Response<Body> {
    match Response::builder().status(status).body(Body::from(body)) {
        Ok(response) => response,
        Err(error_value) => {
            error!(error = %error_value, "Failed to build WebAssembly text response");
            Response::new(Body::from(body))
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderMap, HeaderValue, header};

    use super::accepts_gzip;

    fn headers(value: &'static str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(header::ACCEPT_ENCODING, HeaderValue::from_static(value));
        headers
    }

    #[test]
    fn explicit_zero_quality_refuses_gzip() {
        assert!(!accepts_gzip(&headers("br, gzip;q=0")));
        assert!(!accepts_gzip(&headers("*;q=1, gzip;q=0")));
    }

    #[test]
    fn positive_explicit_or_wildcard_quality_accepts_gzip() {
        assert!(accepts_gzip(&headers("br, gzip;q=0.5")));
        assert!(accepts_gzip(&headers("*;q=0.8")));
        assert!(!accepts_gzip(&headers("gzip;q=invalid")));
    }
}
