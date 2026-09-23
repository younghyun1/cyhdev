//! Per-request time bounds for every HTTP route except WebSocket sessions.
//!
//! Two limits apply. A deadline covers the whole handler future, including reading the
//! body, so a slow trickle cannot hold a worker indefinitely. An idle timeout between
//! request body frames fails a stalled upload early, well before a long upload
//! deadline. Upload routes get deadlines sized for their body limits at a slow but
//! usable link; WebSocket upgrades are exempt because their session runs after the
//! 101 response in its own task, bounded by the socket's own caps and timeouts.
//! Response streaming after the head is not covered; hyper's HTTP/2 keep-alive and
//! the connection caps bound clients that stop reading.

use std::time::Duration;

use axum::{
    body::Body,
    extract::Request,
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use tower_http::timeout::TimeoutBody;

/// Time bounds chosen for a request by method and path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RequestBudget {
    Ordinary,
    /// Superuser operations that call Minecraft, S3, or bulk database work.
    Administrative,
    /// Single-file uploads with the 150 MiB default body limit.
    Upload,
    /// The 1 GiB batch photograph upload.
    BatchUpload,
    /// WebSocket upgrades.
    Unbounded,
}

/// Deadline for the handler future and idle limit between body frames.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RequestLimits {
    pub deadline: Duration,
    pub body_idle: Duration,
}

impl RequestBudget {
    pub fn classify(method: &Method, path: &str, headers: &HeaderMap) -> Self {
        if headers.contains_key(header::UPGRADE) || path.starts_with("/ws/") {
            return Self::Unbounded;
        }
        if method == Method::POST {
            if path == "/api/photographs/batch-upload" {
                return Self::BatchUpload;
            }
            let wasm_assets = path
                .strip_prefix("/api/wasm-modules/")
                .is_some_and(|rest| rest.ends_with("/assets"));
            if wasm_assets
                || matches!(
                    path,
                    "/api/photographs/upload"
                        | "/api/user/upload-profile-picture"
                        | "/api/wasm-modules"
                )
            {
                return Self::Upload;
            }
        }
        if path.starts_with("/api/admin/") {
            return Self::Administrative;
        }
        Self::Ordinary
    }

    pub const fn limits(self) -> Option<RequestLimits> {
        const ORDINARY_IDLE: Duration = Duration::from_secs(30);
        const UPLOAD_IDLE: Duration = Duration::from_secs(60);
        match self {
            Self::Ordinary => Some(RequestLimits {
                deadline: Duration::from_secs(30),
                body_idle: ORDINARY_IDLE,
            }),
            Self::Administrative => Some(RequestLimits {
                deadline: Duration::from_secs(120),
                body_idle: ORDINARY_IDLE,
            }),
            // 150 MiB in 15 minutes needs about 1.4 Mbit/s.
            Self::Upload => Some(RequestLimits {
                deadline: Duration::from_secs(15 * 60),
                body_idle: UPLOAD_IDLE,
            }),
            // 1 GiB in an hour needs about 2.4 Mbit/s.
            Self::BatchUpload => Some(RequestLimits {
                deadline: Duration::from_secs(60 * 60),
                body_idle: UPLOAD_IDLE,
            }),
            Self::Unbounded => None,
        }
    }
}

/// Applies the budget for this request's route.
pub async fn enforce_request_deadline(request: Request, next: Next) -> Response {
    let budget = RequestBudget::classify(request.method(), request.uri().path(), request.headers());
    match budget.limits() {
        Some(limits) => bounded(limits, budget, request, next).await,
        None => next.run(request).await,
    }
}

/// Runs the rest of the stack under `limits`; separate for tests with short limits.
pub(crate) async fn bounded(
    limits: RequestLimits,
    budget: RequestBudget,
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_owned();
    let request = request.map(|body| Body::new(TimeoutBody::new(limits.body_idle, body)));
    match tokio::time::timeout(limits.deadline, next.run(request)).await {
        Ok(response) => response,
        Err(_) => {
            tracing::warn!(
                method = %method,
                path = %path,
                budget = ?budget,
                deadline_ms = limits.deadline.as_millis(),
                "Request exceeded its deadline"
            );
            let mut response =
                (StatusCode::SERVICE_UNAVAILABLE, "Request timed out").into_response();
            response
                .headers_mut()
                .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
            response
        }
    }
}

#[cfg(test)]
#[path = "request_deadline_tests.rs"]
mod tests;
