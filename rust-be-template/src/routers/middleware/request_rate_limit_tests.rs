use std::{
    error::Error,
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use axum::{
    Router,
    http::{HeaderValue, StatusCode, header},
    middleware::from_fn_with_state,
    routing::{get, post},
};
use reqwest::Client;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::oneshot,
    task::JoinHandle,
};

use super::{apply, apply_with_origins, minecraft_exempt};
use crate::{
    routers::middleware::browser_policy::{
        BrowserPolicyConfig, BrowserSecurityPolicy, apply_browser_security_headers,
    },
    util::request_rate_limit::RequestRateLimiter,
};

type TestResult = Result<(), Box<dyn Error>>;

struct Server {
    origin: String,
    shutdown: oneshot::Sender<()>,
    task: JoinHandle<Result<(), std::io::Error>>,
}

impl Server {
    async fn stop(self) -> TestResult {
        let _ = self.shutdown.send(());
        self.task.await??;
        Ok(())
    }
}

async fn serve(router: Router, connect_info: bool) -> Result<Server, Box<dyn Error>> {
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0)).await?;
    let origin = format!("http://{}", listener.local_addr()?);
    let (shutdown, stopped) = oneshot::channel();
    let task = tokio::spawn(async move {
        let shutdown = async move {
            let _ = stopped.await;
        };
        if connect_info {
            axum::serve(
                listener,
                router.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .with_graceful_shutdown(shutdown)
            .await
        } else {
            axum::serve(listener, router)
                .with_graceful_shutdown(shutdown)
                .await
        }
    });
    Ok(Server {
        origin,
        shutdown,
        task,
    })
}

async fn exhausted() -> Arc<RequestRateLimiter> {
    let limiter = Arc::new(RequestRateLimiter::new());
    // Freeze an exhausted bucket ahead of wall time, avoiding refill races over HTTP.
    let now = Instant::now() + Duration::from_secs(30);
    for _ in 0..1024 {
        assert!(limiter.check_at([127, 0, 0, 1].into(), now).await.is_ok());
    }
    limiter
}

#[test]
fn only_segment_delimited_minecraft_namespaces_are_exempt() {
    for path in [
        "/minecraft",
        "/minecraft/map/tiles/world/0/0.png",
        "/api/minecraft/map/seed-tile.bin",
        "/api/minecraft/map/query",
        "/api/admin/minecraft",
        "/api/admin/minecraft/map/waypoints/id",
    ] {
        assert!(minecraft_exempt(path), "{path}");
    }
    for path in [
        "/",
        "/assets/main.js",
        "/swagger-ui/",
        "/api/healthcheck/state",
        "/api/minecraft-extra/map/query",
        "/api/admin/minecraft-other",
        "/minecraftish",
        "/minecraft%2fmap",
        "/api/auth/login",
    ] {
        assert!(!minecraft_exempt(path), "{path}");
    }
}

#[tokio::test]
async fn all_site_surfaces_and_redirects_share_the_exhausted_budget() -> TestResult {
    let limiter = exhausted().await;
    let router = Router::new()
        .route("/api/healthcheck/state", get(|| async { "health" }))
        .route("/swagger-ui/", get(|| async { "docs" }))
        .route("/api-docs/openapi.json", get(|| async { "schema" }))
        .fallback(|| async { "static or fallback" });
    let policy = Arc::new(BrowserSecurityPolicy::new(&BrowserPolicyConfig {
        app_origin: "https://example.test".to_owned(),
        media_origins: Vec::new(),
        inline_script_hashes: Vec::new(),
        strict_transport_security: true,
    })?);
    let site = serve(
        apply_with_origins(
            router,
            Arc::clone(&limiter),
            &[HeaderValue::from_static("https://example.test")],
        )
        .layer(from_fn_with_state(policy, apply_browser_security_headers)),
        true,
    )
    .await?;
    let redirect = serve(
        apply(
            Router::new().fallback(|| async { StatusCode::PERMANENT_REDIRECT }),
            limiter,
        ),
        true,
    )
    .await?;
    let client = Client::new();
    for path in [
        "/api/healthcheck/state",
        "/swagger-ui/",
        "/api-docs/openapi.json",
        "/assets/main.js",
        "/missing",
    ] {
        let response = client
            .get(format!("{}{path}", site.origin))
            .header(header::ORIGIN, "https://example.test")
            .send()
            .await?;
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS, "{path}");
        assert!(response.headers().contains_key(header::RETRY_AFTER));
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        assert_eq!(
            response.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN],
            "https://example.test"
        );
        assert_eq!(
            response.headers()[header::ACCESS_CONTROL_ALLOW_CREDENTIALS],
            "true"
        );
        assert_eq!(
            response.headers()[header::ACCESS_CONTROL_EXPOSE_HEADERS],
            "retry-after"
        );
        assert!(
            response
                .headers()
                .contains_key(header::CONTENT_SECURITY_POLICY)
        );
        let body: serde_json::Value = serde_json::from_slice(&response.bytes().await?)?;
        assert_eq!(body["error_code"], 111);
    }
    let wrong_method = client
        .post(format!("{}/api/healthcheck/state", site.origin))
        .header(header::ORIGIN, "https://untrusted.test")
        .send()
        .await?;
    assert_eq!(wrong_method.status(), StatusCode::TOO_MANY_REQUESTS);
    assert!(
        !wrong_method
            .headers()
            .contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN)
    );
    let redirected = client.get(format!("{}/", redirect.origin)).send().await?;
    assert_eq!(redirected.status(), StatusCode::TOO_MANY_REQUESTS);
    site.stop().await?;
    redirect.stop().await
}

#[tokio::test]
async fn minecraft_bypasses_only_global_admission() -> TestResult {
    let router = Router::new()
        .route("/api/minecraft/map/query", post(|| async { "tile" }))
        .route(
            "/api/admin/minecraft",
            get(|| async { StatusCode::FORBIDDEN }),
        )
        .fallback(|| async { "map asset" });
    let server = serve(apply(router, exhausted().await), true).await?;
    let client = Client::new();
    for _ in 0..10 {
        let response = client
            .post(format!("{}/api/minecraft/map/query", server.origin))
            .send()
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
    }
    let asset = client
        .get(format!(
            "{}/minecraft/map/tiles/world/0/0.png",
            server.origin
        ))
        .send()
        .await?;
    assert_eq!(asset.status(), StatusCode::OK);
    let admin = client
        .get(format!("{}/api/admin/minecraft", server.origin))
        .send()
        .await?;
    assert_eq!(admin.status(), StatusCode::FORBIDDEN);
    for path in [
        "/minecraftish",
        "/api/minecraft-extra",
        "/api/admin/minecraft-other",
    ] {
        let response = client
            .get(format!("{}{path}", server.origin))
            .send()
            .await?;
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    }
    server.stop().await
}

#[tokio::test]
async fn spoofed_forwarded_headers_cannot_reset_the_socket_budget() -> TestResult {
    let server = serve(
        apply(
            Router::new().fallback(|| async { "site" }),
            exhausted().await,
        ),
        true,
    )
    .await?;
    let client = Client::new();
    for ip in ["192.0.2.1", "198.51.100.4", "2001:db8::1"] {
        let response = client
            .get(format!("{}/", server.origin))
            .header("x-forwarded-for", ip)
            .send()
            .await?;
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    }
    server.stop().await
}

#[tokio::test]
async fn rejection_precedes_body_reading_and_feature_work() -> TestResult {
    let reached = Arc::new(AtomicBool::new(false));
    let handler_reached = Arc::clone(&reached);
    let router = Router::new().fallback(move || {
        let reached = Arc::clone(&handler_reached);
        async move {
            reached.store(true, Ordering::Relaxed);
            "handler"
        }
    });
    let server = serve(apply(router, exhausted().await), true).await?;
    let mut stream = TcpStream::connect(server.origin.trim_start_matches("http://")).await?;
    stream.write_all(b"POST /api/auth/login HTTP/1.1\r\nHost: localhost\r\nContent-Length: 1000000\r\nConnection: close\r\n\r\nx").await?;
    let mut response = [0_u8; 1024];
    let count = tokio::time::timeout(Duration::from_secs(2), stream.read(&mut response)).await??;
    assert!(String::from_utf8_lossy(&response[..count]).starts_with("HTTP/1.1 429"));
    assert!(!reached.load(Ordering::Relaxed));
    drop(stream);
    server.stop().await
}

#[tokio::test]
async fn missing_connection_information_fails_closed() -> TestResult {
    let router = Router::new().fallback(|| async { "site" });
    let server = serve(apply(router, Arc::new(RequestRateLimiter::new())), false).await?;
    let response = reqwest::get(format!("{}/", server.origin)).await?;
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    server.stop().await
}
