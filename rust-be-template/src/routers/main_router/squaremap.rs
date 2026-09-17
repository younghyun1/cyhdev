//! Filesystem-backed squaremap assets, isolated from session and API middleware.

use std::path::PathBuf;

use axum::{
    Router,
    extract::Request,
    http::{HeaderValue, StatusCode, header},
    middleware::{Next, from_fn},
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use tower_http::services::ServeDir;

/// Read the public web root once at startup; an absent setting leaves a useful placeholder.
pub(super) fn from_environment() -> anyhow::Result<Router> {
    let root = match std::env::var_os("SQUAREMAP_WEB_DIR") {
        Some(value) if !value.is_empty() => {
            let path = PathBuf::from(value);
            anyhow::ensure!(path.is_absolute(), "SQUAREMAP_WEB_DIR must be absolute");
            Some(path)
        }
        Some(_) => anyhow::bail!("SQUAREMAP_WEB_DIR must not be empty"),
        None => None,
    };
    Ok(router(root))
}

/// Mount only the plugin's public web directory, never the Minecraft server directory.
fn router(root: Option<PathBuf>) -> Router {
    let files = match root {
        Some(root) => Router::new().fallback_service(ServeDir::new(root)),
        None => Router::new().fallback(unavailable),
    };
    Router::new()
        .route(
            "/minecraft/map",
            get(|| async { Redirect::permanent("/minecraft/map/") }),
        )
        .nest("/minecraft/map/", files)
        .layer(from_fn(cache_headers))
}

/// Keep setup failures readable inside the map frame without serving the SPA recursively.
async fn unavailable() -> impl IntoResponse {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        "The Minecraft map is not available yet.",
    )
}

/// Squaremap rewrites stable filenames; immutable caching would freeze map updates.
async fn cache_headers(request: Request, next: Next) -> Response {
    let live_data = request.uri().path().ends_with(".json");
    let mut response = next.run(request).await;
    let policy = if live_data
        || response.status().is_client_error()
        || response.status().is_server_error()
    {
        "no-store"
    } else {
        "public, no-cache"
    };
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static(policy));
    response.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    response
}

#[cfg(test)]
#[path = "squaremap_tests.rs"]
mod tests;
