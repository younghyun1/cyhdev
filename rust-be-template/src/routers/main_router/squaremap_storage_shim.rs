//! Web Storage shim for the sandboxed squaremap document.
//!
//! squaremap's layer control calls `window.localStorage` without a guard. The map now
//! runs in an opaque-origin CSP sandbox, where that access throws and would abort
//! overlay setup, so every squaremap HTML document gets a small in-memory
//! `localStorage` before its module scripts run. Layer visibility then lasts for one
//! page view instead of across visits.
//!
//! The injected document differs from the file on disk, so file validators are
//! neither forwarded nor honored for HTML; the documents are a few kilobytes.

use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};

pub(super) const STORAGE_SHIM: &str = include_str!("squaremap_storage_shim.js");
/// squaremap's index page is about 2 KiB; anything far larger is not its HTML shell.
const MAX_DOCUMENT_BYTES: usize = 1024 * 1024;

/// Injects [`STORAGE_SHIM`] into successful HTML responses.
pub(super) async fn inject(mut request: Request, next: Next) -> Response {
    let document = is_document_path(request.uri().path())
        && matches!(*request.method(), Method::GET | Method::HEAD);
    if document {
        for name in [header::IF_NONE_MATCH, header::IF_MODIFIED_SINCE] {
            request.headers_mut().remove(name);
        }
    }
    let head = request.method() == Method::HEAD;
    let response = next.run(request).await;
    if !document || response.status() != StatusCode::OK || !is_html(response.headers()) {
        return response;
    }
    let (mut parts, body) = response.into_parts();
    for name in [header::CONTENT_LENGTH, header::ETAG, header::LAST_MODIFIED] {
        parts.headers.remove(name);
    }
    if head {
        return Response::from_parts(parts, Body::empty());
    }
    let bytes = match axum::body::to_bytes(body, MAX_DOCUMENT_BYTES).await {
        Ok(bytes) => bytes,
        Err(error) => {
            tracing::warn!(error = %error, "Squaremap document could not be prepared");
            let mut failure = (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Map document unavailable",
            )
                .into_response();
            failure
                .headers_mut()
                .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
            return failure;
        }
    };
    let injected = with_shim(&String::from_utf8_lossy(&bytes));
    Response::from_parts(parts, Body::from(injected))
}

/// Inserts the shim right after the opening `<head>` tag, or first if there is none.
pub(super) fn with_shim(document: &str) -> String {
    let script = format!("<script>{STORAGE_SHIM}</script>");
    let lower = document.to_ascii_lowercase();
    let insert_at = lower
        .find("<head")
        .filter(|start| {
            // Exclude `<header>` and similar elements that only share the prefix.
            matches!(
                lower.as_bytes().get(start + 5),
                Some(b'>' | b' ' | b'\t' | b'\n' | b'\r' | b'/')
            )
        })
        .and_then(|start| lower[start..].find('>').map(|end| start + end + 1))
        .unwrap_or(0);
    let mut injected = String::with_capacity(document.len() + script.len());
    injected.push_str(&document[..insert_at]);
    injected.push_str(&script);
    injected.push_str(&document[insert_at..]);
    injected
}

fn is_document_path(path: &str) -> bool {
    path.ends_with('/') || path.ends_with(".html") || path.ends_with(".htm")
}

fn is_html(headers: &HeaderMap) -> bool {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| {
            value
                .trim_start()
                .to_ascii_lowercase()
                .starts_with("text/html")
        })
}

#[cfg(test)]
mod tests {
    use super::{STORAGE_SHIM, with_shim};

    #[test]
    fn shim_is_placed_before_any_document_script() {
        let document = "<!doctype html><html><HEAD lang=\"en\"><script type=\"module\" src=\"a.js\"></script></head></html>";
        let injected = with_shim(document);
        let shim = injected.find(STORAGE_SHIM);
        let module = injected.find("type=\"module\"");
        assert!(injected.starts_with("<!doctype html><html><HEAD lang=\"en\"><script>"));
        assert!(shim.is_some() && module.is_some() && shim < module);
    }

    #[test]
    fn documents_without_head_get_the_shim_first() {
        let injected = with_shim("<header>Map</header>");
        assert!(injected.starts_with("<script>"));
        assert!(injected.ends_with("</script><header>Map</header>"));
    }
}
