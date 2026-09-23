use std::{error::Error, time::Duration};

use axum::{
    Router,
    body::Bytes,
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    middleware::from_fn,
    routing::{get, post},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time::{Instant, timeout},
};

use super::{RequestBudget, RequestLimits, bounded};

type TestResult = Result<(), Box<dyn Error>>;

const SHORT: RequestLimits = RequestLimits {
    deadline: Duration::from_millis(300),
    body_idle: Duration::from_millis(150),
};

#[test]
fn routes_are_classified_by_method_and_path() {
    let none = HeaderMap::new();
    let classify = |method: Method, path: &str| RequestBudget::classify(&method, path, &none);
    assert_eq!(
        classify(Method::GET, "/api/blog/posts"),
        RequestBudget::Ordinary
    );
    assert_eq!(
        classify(Method::GET, "/assets/app.js"),
        RequestBudget::Ordinary
    );
    assert_eq!(
        classify(Method::GET, "/ws/live-chat"),
        RequestBudget::Unbounded
    );
    assert_eq!(
        classify(Method::POST, "/api/photographs/batch-upload"),
        RequestBudget::BatchUpload
    );
    for path in [
        "/api/photographs/upload",
        "/api/user/upload-profile-picture",
        "/api/wasm-modules",
        "/api/wasm-modules/0199aaaa-0000-7000-8000-000000000000/assets",
    ] {
        assert_eq!(
            classify(Method::POST, path),
            RequestBudget::Upload,
            "{path}"
        );
    }
    // Only the upload methods receive the long budget.
    assert_eq!(
        classify(Method::GET, "/api/wasm-modules"),
        RequestBudget::Ordinary
    );
    assert_eq!(
        classify(Method::GET, "/api/photographs/batch-upload"),
        RequestBudget::Ordinary
    );
    assert_eq!(
        classify(Method::POST, "/api/admin/sync-i18n-cache"),
        RequestBudget::Administrative
    );

    let mut upgrade = HeaderMap::new();
    upgrade.insert(header::UPGRADE, HeaderValue::from_static("websocket"));
    assert_eq!(
        RequestBudget::classify(&Method::GET, "/api/anything", &upgrade),
        RequestBudget::Unbounded
    );
    assert_eq!(RequestBudget::Unbounded.limits(), None);
}

#[test]
fn upload_budgets_outlast_ordinary_requests() {
    let ordinary = RequestBudget::Ordinary
        .limits()
        .map(|limits| limits.deadline);
    let upload = RequestBudget::Upload.limits().map(|limits| limits.deadline);
    let batch = RequestBudget::BatchUpload
        .limits()
        .map(|limits| limits.deadline);
    assert!(ordinary < upload && upload < batch);
}

async fn serve(app: Router) -> Result<String, Box<dyn Error>> {
    let app = app.layer(from_fn(|request, next| {
        bounded(SHORT, RequestBudget::Ordinary, request, next)
    }));
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await?;
    let origin = format!("127.0.0.1:{}", listener.local_addr()?.port());
    tokio::spawn(async move { axum::serve(listener, app).await });
    Ok(origin)
}

#[tokio::test]
async fn slow_handler_receives_service_unavailable_at_the_deadline() -> TestResult {
    let app = Router::new().route(
        "/slow",
        get(|| async {
            tokio::time::sleep(Duration::from_secs(10)).await;
            "late"
        }),
    );
    let origin = serve(app).await?;
    let started = Instant::now();
    let response = reqwest::get(format!("http://{origin}/slow")).await?;
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    assert!(started.elapsed() < Duration::from_secs(5));
    Ok(())
}

#[tokio::test]
async fn stalled_request_body_fails_before_the_deadline() -> TestResult {
    let app = Router::new().route(
        "/upload",
        post(|body: Bytes| async move { body.len().to_string() }),
    );
    let origin = serve(app).await?;
    let mut stream = TcpStream::connect(&origin).await?;
    // Announce ten body bytes, send one, then stall.
    stream
        .write_all(b"POST /upload HTTP/1.1\r\nHost: localhost\r\nContent-Length: 10\r\n\r\nx")
        .await?;
    let started = Instant::now();
    let mut response = vec![0_u8; 256];
    let read = timeout(Duration::from_secs(5), stream.read(&mut response)).await??;
    let response = String::from_utf8_lossy(&response[..read]);
    assert!(
        response.starts_with("HTTP/1.1 400") || response.starts_with("HTTP/1.1 408"),
        "unexpected response: {response}"
    );
    assert!(
        started.elapsed() < SHORT.deadline,
        "idle timeout should fire first"
    );
    Ok(())
}

#[tokio::test]
async fn fast_requests_pass_through_unchanged() -> TestResult {
    let app = Router::new().route(
        "/upload",
        post(|body: Bytes| async move { body.len().to_string() }),
    );
    let origin = serve(app).await?;
    let response = reqwest::Client::new()
        .post(format!("http://{origin}/upload"))
        .body("0123456789")
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.text().await?, "10");
    Ok(())
}
