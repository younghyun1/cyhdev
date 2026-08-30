use std::{error::Error, sync::Arc};

use axum::{
    Json, Router,
    http::{StatusCode, header},
    middleware::{from_fn, from_fn_with_state},
    routing::post,
};

use super::auth_abuse::{enforce_auth_ip_throttle, sensitive_auth_response_headers};
use crate::features::accounts::service::auth_abuse::AuthAbuseService;

#[tokio::test]
async fn auth_route_headers_cover_extractor_and_throttle_rejections(
) -> Result<(), Box<dyn Error>> {
    let limiter = Arc::new(AuthAbuseService::new()?);
    let router = Router::new()
        .route("/api/auth/login", post(accept_json))
        .layer(from_fn_with_state(limiter, enforce_auth_ip_throttle))
        .layer(from_fn(sensitive_auth_response_headers));
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await?;
    let address = listener.local_addr()?;
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let service = router.into_make_service_with_connect_info::<std::net::SocketAddr>();
    let server = tokio::spawn(async move {
        axum::serve(listener, service)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.await;
            })
            .await
    });
    let client = reqwest::Client::new();
    let url = format!("http://{address}/api/auth/login");

    let malformed = client
        .post(&url)
        .header(header::CONTENT_TYPE, "application/json")
        .body("{")
        .send()
        .await?;
    assert!(malformed.status().is_client_error());
    assert_private_headers(malformed.headers());

    for _ in 0..9 {
        let admitted = client
            .post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .body("{}")
            .send()
            .await?;
        assert_eq!(admitted.status(), StatusCode::NO_CONTENT);
    }
    let throttled = client
        .post(&url)
        .header(header::CONTENT_TYPE, "application/json")
        .body("{}")
        .send()
        .await?;
    assert_eq!(throttled.status(), StatusCode::TOO_MANY_REQUESTS);
    assert!(throttled.headers().contains_key(header::RETRY_AFTER));
    assert_private_headers(throttled.headers());

    let _ = shutdown_tx.send(());
    server.await??;
    Ok(())
}

async fn accept_json(Json(_body): Json<serde_json::Value>) -> StatusCode {
    StatusCode::NO_CONTENT
}

fn assert_private_headers(headers: &reqwest::header::HeaderMap) {
    assert_eq!(
        headers.get(header::CACHE_CONTROL),
        Some(&reqwest::header::HeaderValue::from_static("no-store, max-age=0")),
    );
    assert_eq!(
        headers.get(header::REFERRER_POLICY),
        Some(&reqwest::header::HeaderValue::from_static("no-referrer")),
    );
}
