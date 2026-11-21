//! Router-boundary tests behind the same guards the root router applies.

use axum::http::{StatusCode, header};
use serde_json::Value;

use super::test_harness::{TRUSTED_ORIGIN, harness};
use crate::features::kcna_archive::service::test_snapshot::{KOREAN_ARTICLE_ID, write_snapshot};

/// Reader routes; closed to a member until a superuser lists the account.
const READER_ROUTES: [&str; 5] = [
    "/api/kcna-archive/search?q=peace",
    "/api/kcna-archive/histogram",
    "/api/kcna-archive/trends?term=peace",
    "/api/kcna-archive/documents/00000000-0000-0000-0000-000000000001",
    "/api/kcna-archive/export",
];

/// Routes that stay with superusers whoever is listed.
const ADMIN_ROUTES: [(&str, &str); 5] = [
    ("GET", "/api/admin/kcna-archive/status"),
    ("POST", "/api/admin/kcna-archive/reload"),
    ("GET", "/api/admin/kcna-archive/readers"),
    ("POST", "/api/admin/kcna-archive/readers"),
    (
        "DELETE",
        "/api/admin/kcna-archive/readers/00000000-0000-0000-0000-000000000001",
    ),
];

#[tokio::test]
async fn every_route_rejects_missing_sessions_other_roles_and_foreign_origins() -> anyhow::Result<()>
{
    let directory = tempfile::tempdir()?;
    write_snapshot(directory.path())?;
    let harness = harness(directory.path()).await?;
    let routes = READER_ROUTES
        .iter()
        .map(|path| ("GET", *path))
        .chain(ADMIN_ROUTES);
    for (method, path) in routes {
        let method = reqwest::Method::from_bytes(method.as_bytes())?;
        let cases = [
            (None, StatusCode::UNAUTHORIZED),
            (Some(harness.user.token.as_str()), StatusCode::FORBIDDEN),
            (
                Some(harness.moderator.token.as_str()),
                StatusCode::FORBIDDEN,
            ),
            // The session check turns an unverified address away before any gate.
            (
                Some(harness.unverified.token.as_str()),
                StatusCode::BAD_REQUEST,
            ),
        ];
        for (token, expected) in cases {
            let response = harness
                .request(method.clone(), path, token, Some(TRUSTED_ORIGIN))
                .send()
                .await?;
            assert_eq!(response.status(), expected, "{method} {path}");
            assert_eq!(
                response.headers()[header::CACHE_CONTROL],
                "no-store, max-age=0",
                "{method} {path}"
            );
        }
    }
    // The state-changing routes also need a trusted browser origin.
    for (method, path) in [
        (reqwest::Method::POST, "/api/admin/kcna-archive/reload"),
        (reqwest::Method::POST, "/api/admin/kcna-archive/readers"),
        (
            reqwest::Method::DELETE,
            "/api/admin/kcna-archive/readers/00000000-0000-0000-0000-000000000001",
        ),
    ] {
        for origin in [Some("https://untrusted.example"), None] {
            let response = harness
                .request(method.clone(), path, Some(&harness.admin.token), origin)
                .send()
                .await?;
            assert_eq!(response.status(), StatusCode::FORBIDDEN, "{method} {path}");
        }
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
        .get_json("/api/kcna-archive/search?q=peace&lang=eng&limit=1")
        .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["total"], 2);
    assert_eq!(body["data"]["items"][0]["language"], "eng");
    assert!(body["data"]["items"][0]["publicationDate"].is_string());

    let (status, body) = harness
        .get_json("/api/kcna-archive/trends?term=peace&term=trade&bucket=year")
        .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["series"].as_array().map(Vec::len), Some(2));

    let (status, body) = harness
        .get_json(&format!(
            "/api/kcna-archive/documents/{KOREAN_ARTICLE_ID}?q=%ED%95%B5"
        ))
        .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["publicationPrecision"], "day");

    let (status, body) = harness
        .get_json("/api/kcna-archive/documents/00000000-0000-0000-0000-0000000003e7")
        .await?;
    assert_eq!(
        (status, &body["error_code"]),
        (StatusCode::NOT_FOUND, &Value::from(112))
    );

    // Five terms exceed the limit of four.
    let (status, _) = harness
        .get_json("/api/kcna-archive/trends?term=a&term=b&term=c&term=d&term=e")
        .await?;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = harness
        .get_json("/api/kcna-archive/export?format=xml")
        .await?;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let export = harness
        .request(
            reqwest::Method::GET,
            "/api/kcna-archive/export?q=peace&format=jsonl",
            Some(&harness.admin.token),
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
            Some(&harness.admin.token),
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

    let (status, body) = harness.get_json("/api/kcna-archive/search?q=peace").await?;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["error_code"], 113);
    Ok(())
}
