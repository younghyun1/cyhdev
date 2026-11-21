//! Router-boundary tests behind the same guards the root router applies.

use std::{net::SocketAddr, path::Path, sync::Arc};

use axum::{
    Extension,
    http::{StatusCode, header},
    middleware::{from_fn, from_fn_with_state},
};
use kcna_archive_core::service::archive::Archive;
use serde_json::Value;
use tokio::task::JoinHandle;
use uuid::Uuid;

use super::routes::routes;
use crate::{
    features::{
        accounts::{
            domain::{account::LoginAccount, role::RoleType, session::SESSION_COOKIE_NAME},
            service::session_service::SessionService,
        },
        kcna_archive::service::{
            archive_service::KcnaArchiveService,
            state::ServiceState,
            test_snapshot::{KOREAN_ARTICLE_ID, write_snapshot},
        },
    },
    init::state::{DeploymentEnvironment, PublicAppOrigin},
    routers::middleware::{
        auth::auth_middleware,
        role::require_superuser_middleware,
        sensitive_response::sensitive_admin_response_headers,
        trusted_origin::{TrustedOrigins, require_trusted_origin},
    },
};

const TRUSTED_ORIGIN: &str = "https://cyhdev.com";

struct Harness {
    address: SocketAddr,
    admin: String,
    user: String,
    server: JoinHandle<std::io::Result<()>>,
    client: reqwest::Client,
}

impl Drop for Harness {
    fn drop(&mut self) {
        self.server.abort();
    }
}

async fn harness(directory: &Path) -> anyhow::Result<Harness> {
    let service = Arc::new(
        KcnaArchiveService::start(directory.to_owned(), Arc::new(Archive::load_into_memory)).await,
    );
    // Wait for the first load so every test starts from a settled state.
    for _ in 0..1000 {
        if service.status().await?.state != ServiceState::Loading {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    let sessions = Arc::new(SessionService::new());
    let account = LoginAccount {
        user_id: Uuid::new_v4(),
        user_name: "archive-test".into(),
        password_hash: "unused".into(),
        is_email_verified: true,
        country: 840,
        language: 1,
    };
    let user = sessions
        .create(&account, RoleType::User, None, None)
        .await?;
    let admin = sessions
        .create(&account, RoleType::Younghyun, None, None)
        .await?;
    let origin = PublicAppOrigin::parse(Some(TRUSTED_ORIGIN), DeploymentEnvironment::Prod)?;
    let origins = Arc::new(TrustedOrigins::from_environment(
        DeploymentEnvironment::Prod,
        &origin,
    )?);
    let router = routes::<()>()
        .layer(Extension(service))
        .layer(from_fn(require_superuser_middleware))
        .layer(from_fn_with_state(sessions, auth_middleware))
        .layer(from_fn_with_state(origins, require_trusted_origin))
        .layer(from_fn(sensitive_admin_response_headers));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let server = tokio::spawn(async move { axum::serve(listener, router).await });
    Ok(Harness {
        address,
        admin: admin.expose().to_owned(),
        user: user.expose().to_owned(),
        server,
        client: reqwest::Client::new(),
    })
}

impl Harness {
    fn request(
        &self,
        method: reqwest::Method,
        path: &str,
        token: Option<&str>,
        origin: Option<&str>,
    ) -> reqwest::RequestBuilder {
        let mut request = self
            .client
            .request(method, format!("http://{}{path}", self.address));
        if let Some(token) = token {
            request = request.header(header::COOKIE, format!("{SESSION_COOKIE_NAME}={token}"));
        }
        if let Some(origin) = origin {
            request = request.header(header::ORIGIN, origin);
        }
        request
    }

    async fn get_json(&self, path: &str) -> anyhow::Result<(StatusCode, Value)> {
        let response = self
            .request(reqwest::Method::GET, path, Some(&self.admin), None)
            .send()
            .await?;
        let status = response.status();
        Ok((status, serde_json::from_str(&response.text().await?)?))
    }
}

const ALL_ROUTES: [(&str, &str); 7] = [
    ("GET", "/api/admin/kcna-archive/search?q=peace"),
    ("GET", "/api/admin/kcna-archive/histogram"),
    ("GET", "/api/admin/kcna-archive/trends?term=peace"),
    ("GET", "/api/admin/kcna-archive/status"),
    (
        "GET",
        "/api/admin/kcna-archive/documents/00000000-0000-0000-0000-000000000001",
    ),
    ("GET", "/api/admin/kcna-archive/export"),
    ("POST", "/api/admin/kcna-archive/reload"),
];

#[tokio::test]
async fn every_route_rejects_missing_sessions_non_superusers_and_foreign_origins()
-> anyhow::Result<()> {
    let directory = tempfile::tempdir()?;
    write_snapshot(directory.path())?;
    let harness = harness(directory.path()).await?;
    for (method, path) in ALL_ROUTES {
        let method = reqwest::Method::from_bytes(method.as_bytes())?;
        let cases = [
            (None, Some(TRUSTED_ORIGIN), StatusCode::UNAUTHORIZED),
            (
                Some(harness.user.as_str()),
                Some(TRUSTED_ORIGIN),
                StatusCode::FORBIDDEN,
            ),
        ];
        for (token, origin, expected) in cases {
            let response = harness
                .request(method.clone(), path, token, origin)
                .send()
                .await?;
            assert_eq!(response.status(), expected, "{method} {path}");
            assert_eq!(
                response.headers()[header::CACHE_CONTROL],
                "no-store, max-age=0"
            );
        }
    }
    // The state-changing route also needs a trusted browser origin.
    for origin in [Some("https://untrusted.example"), None] {
        let response = harness
            .request(
                reqwest::Method::POST,
                "/api/admin/kcna-archive/reload",
                Some(&harness.admin),
                origin,
            )
            .send()
            .await?;
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
    Ok(())
}

#[tokio::test]
async fn a_superuser_reads_the_snapshot_through_camel_case_envelopes() -> anyhow::Result<()> {
    let directory = tempfile::tempdir()?;
    write_snapshot(directory.path())?;
    let harness = harness(directory.path()).await?;

    let (status, body) = harness.get_json("/api/admin/kcna-archive/status").await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["state"], "ready");
    assert_eq!(body["data"]["snapshot"]["indexed"], 3);

    let (status, body) = harness
        .get_json("/api/admin/kcna-archive/search?q=peace&lang=eng&limit=1")
        .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["total"], 2);
    assert_eq!(body["data"]["items"][0]["language"], "eng");
    assert!(body["data"]["items"][0]["publicationDate"].is_string());

    let (status, body) = harness
        .get_json("/api/admin/kcna-archive/trends?term=peace&term=trade&bucket=year")
        .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["series"].as_array().map(Vec::len), Some(2));

    let (status, body) = harness
        .get_json(&format!(
            "/api/admin/kcna-archive/documents/{KOREAN_ARTICLE_ID}?q=%ED%95%B5"
        ))
        .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["publicationPrecision"], "day");

    let (status, body) = harness
        .get_json("/api/admin/kcna-archive/documents/00000000-0000-0000-0000-0000000003e7")
        .await?;
    assert_eq!(
        (status, &body["error_code"]),
        (StatusCode::NOT_FOUND, &Value::from(112))
    );

    // Five terms exceed the limit of four.
    let (status, _) = harness
        .get_json("/api/admin/kcna-archive/trends?term=a&term=b&term=c&term=d&term=e")
        .await?;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = harness
        .get_json("/api/admin/kcna-archive/export?format=xml")
        .await?;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let export = harness
        .request(
            reqwest::Method::GET,
            "/api/admin/kcna-archive/export?q=peace&format=jsonl",
            Some(&harness.admin),
            None,
        )
        .send()
        .await?;
    assert_eq!(export.status(), StatusCode::OK);
    assert_eq!(
        export.headers()[header::CONTENT_TYPE],
        "application/x-ndjson"
    );
    assert_eq!(
        export.headers()[header::CONTENT_DISPOSITION],
        "attachment; filename=\"kcna-archive-export.jsonl\""
    );
    assert_eq!(
        export.headers()[header::CACHE_CONTROL],
        "no-store, max-age=0"
    );
    assert_eq!(export.text().await?.lines().count(), 2);

    let reload = harness
        .request(
            reqwest::Method::POST,
            "/api/admin/kcna-archive/reload",
            Some(&harness.admin),
            Some(TRUSTED_ORIGIN),
        )
        .send()
        .await?;
    assert_eq!(reload.status(), StatusCode::OK);
    Ok(())
}

#[tokio::test]
async fn without_a_snapshot_status_is_200_and_data_routes_are_503() -> anyhow::Result<()> {
    let directory = tempfile::tempdir()?;
    let harness = harness(directory.path()).await?;

    let (status, body) = harness.get_json("/api/admin/kcna-archive/status").await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["state"], "not_installed");
    assert_eq!(body["data"]["snapshot"], Value::Null);

    let (status, body) = harness
        .get_json("/api/admin/kcna-archive/search?q=peace")
        .await?;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["error_code"], 113);
    Ok(())
}
