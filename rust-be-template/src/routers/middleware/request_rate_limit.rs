//! Shared request admission before API, static, documentation, or redirect work.

use std::{net::SocketAddr, sync::Arc};

use axum::{
    Router,
    extract::{ConnectInfo, Request, State},
    http::{HeaderValue, StatusCode, header},
    middleware::{Next, from_fn_with_state},
    response::{IntoResponse, Response},
};

use crate::{
    errors::code_error::{CodeError, CodeErrorResp},
    util::{extract::client_ip::extract_client_ip, request_rate_limit::RequestRateLimiter},
};

/// Applies one shared budget to routes and fallbacks, including rejected methods.
pub fn apply(router: Router, limiter: Arc<RequestRateLimiter>) -> Router {
    apply_with_origins(router, limiter, &[])
}

#[derive(Clone)]
struct AdmissionState {
    limiter: Arc<RequestRateLimiter>,
    origins: Arc<[HeaderValue]>,
}

/// Shares the API's origin allowlist so outer admission errors remain readable.
pub(crate) fn apply_with_origins(
    router: Router,
    limiter: Arc<RequestRateLimiter>,
    origins: &[HeaderValue],
) -> Router {
    let policy = AdmissionState {
        limiter,
        origins: Arc::from(origins),
    };
    router.layer(from_fn_with_state(policy, enforce_request_rate_limit))
}

/// Minecraft tile fanout has its own admission bounds and must not debit the site budget.
pub(crate) fn minecraft_exempt(path: &str) -> bool {
    ["/api/minecraft", "/api/admin/minecraft", "/minecraft"]
        .iter()
        .any(|prefix| {
            path == *prefix
                || path
                    .strip_prefix(prefix)
                    .is_some_and(|rest| rest.starts_with('/'))
        })
}

async fn enforce_request_rate_limit(
    State(policy): State<AdmissionState>,
    request: Request,
    next: Next,
) -> Response {
    if minecraft_exempt(request.uri().path()) {
        return next.run(request).await;
    }
    let peer = match request.extensions().get::<ConnectInfo<SocketAddr>>() {
        Some(ConnectInfo(peer)) => *peer,
        None => {
            tracing::error!("Request admission requires socket connection information");
            return no_store(StatusCode::INTERNAL_SERVER_ERROR.into_response());
        }
    };
    let client = match extract_client_ip(request.headers(), peer) {
        Some(client) => client,
        None => peer.ip(),
    };
    let origin = request
        .headers()
        .get(header::ORIGIN)
        .filter(|origin| {
            policy.origins.contains(origin)
                && request.headers().get_all(header::ORIGIN).iter().count() == 1
        })
        .cloned();
    match policy.limiter.check(client).await {
        Ok(()) => next.run(request).await,
        Err(rejection) => {
            let mut response = no_store(
                CodeErrorResp::from(CodeError::REQUEST_THROTTLED)
                    .with_retry_after(rejection.retry_after)
                    .into_response(),
            );
            // The API CorsLayer is downstream; a rejected request never reaches it.
            response
                .headers_mut()
                .insert(header::VARY, HeaderValue::from_static("origin"));
            if let Some(origin) = origin {
                let headers = response.headers_mut();
                headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);
                headers.insert(
                    header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
                    HeaderValue::from_static("true"),
                );
                headers.insert(
                    header::ACCESS_CONTROL_EXPOSE_HEADERS,
                    HeaderValue::from_static("retry-after"),
                );
            }
            response
        }
    }
}

fn no_store(mut response: Response) -> Response {
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

#[cfg(test)]
#[path = "request_rate_limit_tests.rs"]
mod tests;
