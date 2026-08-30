//! HTTP admission and response helpers for authentication-abuse controls.

use std::{net::SocketAddr, sync::Arc};

use axum::{
    body::Body,
    extract::{ConnectInfo, State},
    http::{HeaderValue, Method, Request, Response, header},
    middleware::Next,
    response::IntoResponse,
};

use crate::{
    errors::code_error::{CodeError, CodeErrorResp},
    features::accounts::{
        domain::auth_abuse::{AuthEndpoint, AuthThrottleRejection},
        service::auth_abuse::AuthAbuseService,
    },
    util::extract::client_ip::extract_client_ip,
};

/// Reject an over-budget IP before Axum buffers or deserializes an auth body.
pub async fn enforce_auth_ip_throttle(
    State(service): State<Arc<AuthAbuseService>>,
    ConnectInfo(socket_addr): ConnectInfo<SocketAddr>,
    request: Request<Body>,
    next: Next,
) -> Response<Body> {
    if request.method() != Method::POST {
        return next.run(request).await;
    }
    let endpoint = match AuthEndpoint::from_path(request.uri().path()) {
        Some(endpoint) => endpoint,
        None => return next.run(request).await,
    };
    let client_ip = match extract_client_ip(request.headers(), socket_addr) {
        Some(client_ip) => client_ip,
        None => socket_addr.ip(),
    };
    match service.check_ip(endpoint, client_ip).await {
        Ok(()) => next.run(request).await,
        Err(rejection) => map_auth_throttle_rejection(rejection).into_response(),
    }
}

/// Apply sensitive response policy to successes, extractor failures, and throttles.
pub async fn sensitive_auth_response_headers(
    request: Request<Body>,
    next: Next,
) -> Response<Body> {
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store, max-age=0"),
    );
    response.headers_mut().insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    response
}

pub fn map_auth_throttle_rejection(rejection: AuthThrottleRejection) -> CodeErrorResp {
    let retry_after_seconds = rejection
        .retry_after()
        .as_secs()
        .saturating_add(u64::from(rejection.retry_after().subsec_nanos() > 0))
        .max(1);
    tracing::warn!(
        event = "auth_abuse_rejected",
        endpoint = rejection.endpoint().as_str(),
        dimension = rejection.dimension().as_str(),
        capacity_saturated = rejection.capacity_saturated(),
        retry_after_seconds,
        "Authentication attempt rejected"
    );
    CodeErrorResp::from(CodeError::AUTH_THROTTLED).with_retry_after(rejection.retry_after())
}
