//! The reader list decides who reads, and only a superuser changes it.

use axum::http::StatusCode;
use serde_json::Value;
use uuid::Uuid;

use super::test_harness::{TRUSTED_ORIGIN, harness};
use crate::features::kcna_archive::service::test_snapshot::write_snapshot;

const ACCESS: &str = "/api/kcna-archive/access";
const READERS: &str = "/api/admin/kcna-archive/readers";
const SEARCH: &str = "/api/kcna-archive/search?q=peace";

fn listed(body: &Value) -> Vec<&str> {
    body["data"]["readers"]
        .as_array()
        .map(|readers| {
            readers
                .iter()
                .filter_map(|reader| reader["userId"].as_str())
                .collect()
        })
        .unwrap_or_default()
}

#[tokio::test]
async fn only_listed_accounts_read_and_only_while_they_are_listed() -> anyhow::Result<()> {
    let directory = tempfile::tempdir()?;
    write_snapshot(directory.path())?;
    let harness = harness(directory.path()).await?;
    let (admin, user, moderator) = (&harness.admin, &harness.user, &harness.moderator);

    // Nobody is listed: a member learns so without an error, and the data routes say why.
    let (status, body) = harness.get_as(&user.token, ACCESS).await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["data"],
        serde_json::json!({ "canRead": false, "adminOnly": false })
    );
    let (status, body) = harness.get_as(&user.token, SEARCH).await?;
    assert_eq!(
        (status, &body["error_code"]),
        (StatusCode::FORBIDDEN, &Value::from(119))
    );
    let (status, body) = harness.get_json(ACCESS).await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["data"],
        serde_json::json!({ "canRead": true, "adminOnly": true })
    );
    let (status, body) = harness.get_json(READERS).await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["readers"], serde_json::json!([]));
    assert_eq!(body["data"]["limit"], 500);

    // Listing one account admits that account and nobody else.
    let (status, body) = harness.grant(&admin.token, user.id).await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(listed(&body), [user.id.to_string().as_str()]);
    assert_eq!(body["data"]["readers"][0]["userName"], "member");
    assert_eq!(
        body["data"]["readers"][0]["userEmail"],
        "member@example.test"
    );
    assert!(body["data"]["readers"][0]["grantedAt"].is_string());
    let (status, body) = harness.get_as(&user.token, ACCESS).await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["data"],
        serde_json::json!({ "canRead": true, "adminOnly": false })
    );
    let (status, body) = harness.get_as(&user.token, SEARCH).await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["total"], 2);
    let export = harness
        .request(
            reqwest::Method::GET,
            "/api/kcna-archive/export?q=peace",
            Some(&user.token),
            None,
        )
        .send()
        .await?;
    assert_eq!(export.status(), StatusCode::OK);
    let (status, _) = harness.get_as(&moderator.token, SEARCH).await?;
    assert_eq!(status, StatusCode::FORBIDDEN);
    // The superuser no longer reads alone.
    let (_, body) = harness.get_json(ACCESS).await?;
    assert_eq!(body["data"]["adminOnly"], false);

    // Being listed grants reading only.
    let (status, _) = harness
        .get_as(&user.token, "/api/admin/kcna-archive/status")
        .await?;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = harness.get_as(&user.token, READERS).await?;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = harness.grant(&user.token, moderator.id).await?;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = harness.revoke(&user.token, user.id).await?;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Listing twice changes nothing, and the newest grant comes first.
    let (status, body) = harness.grant(&admin.token, user.id).await?;
    assert_eq!((status, listed(&body).len()), (StatusCode::OK, 1));
    let (_, body) = harness.grant(&admin.token, moderator.id).await?;
    assert_eq!(
        listed(&body),
        [
            moderator.id.to_string().as_str(),
            user.id.to_string().as_str()
        ]
    );

    // Removal takes effect on the next request and is safe to repeat.
    let (status, body) = harness.revoke(&admin.token, user.id).await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(listed(&body), [moderator.id.to_string().as_str()]);
    let (status, _) = harness.get_as(&user.token, SEARCH).await?;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = harness.get_as(&moderator.token, SEARCH).await?;
    assert_eq!(status, StatusCode::OK);
    let (status, body) = harness.revoke(&admin.token, user.id).await?;
    assert_eq!((status, listed(&body).len()), (StatusCode::OK, 1));
    Ok(())
}

#[tokio::test]
async fn an_unverified_address_stays_out_even_when_listed() -> anyhow::Result<()> {
    let directory = tempfile::tempdir()?;
    write_snapshot(directory.path())?;
    let harness = harness(directory.path()).await?;
    let (status, _) = harness
        .grant(&harness.admin.token, harness.unverified.id)
        .await?;
    assert_eq!(status, StatusCode::OK);
    for path in [ACCESS, SEARCH] {
        let (status, body) = harness.get_as(&harness.unverified.token, path).await?;
        assert_eq!(
            (status, &body["error_code"]),
            (StatusCode::BAD_REQUEST, &Value::from(41)),
            "{path}"
        );
    }
    Ok(())
}

#[tokio::test]
async fn an_unknown_account_or_a_malformed_request_changes_nothing() -> anyhow::Result<()> {
    let directory = tempfile::tempdir()?;
    let harness = harness(directory.path()).await?;
    let admin = &harness.admin.token;

    let (status, body) = harness.grant(admin, Uuid::new_v4()).await?;
    assert_eq!(
        (status, &body["error_code"]),
        (StatusCode::NOT_FOUND, &Value::from(121))
    );

    for body in [
        r#"{"userId":"not-an-identifier"}"#,
        r#"{"userId":"00000000-0000-0000-0000-000000000001","extra":true}"#,
        r#"{}"#,
    ] {
        let response = harness
            .request(
                reqwest::Method::POST,
                READERS,
                Some(admin),
                Some(TRUSTED_ORIGIN),
            )
            .header("content-type", "application/json")
            .body(body)
            .send()
            .await?;
        assert!(response.status().is_client_error(), "{body}");
    }
    let malformed = harness
        .request(
            reqwest::Method::DELETE,
            "/api/admin/kcna-archive/readers/not-an-identifier",
            Some(admin),
            Some(TRUSTED_ORIGIN),
        )
        .send()
        .await?;
    assert!(malformed.status().is_client_error());

    let (_, body) = harness.get_json(READERS).await?;
    assert_eq!(body["data"]["readers"], serde_json::json!([]));
    Ok(())
}

#[tokio::test]
async fn a_failed_store_reports_500_and_keeps_the_list_in_force() -> anyhow::Result<()> {
    let directory = tempfile::tempdir()?;
    write_snapshot(directory.path())?;
    let harness = harness(directory.path()).await?;
    let (admin, user, moderator) = (&harness.admin, &harness.user, &harness.moderator);
    harness.grant(&admin.token, user.id).await?;

    harness.store.fail(true);
    for (status, body) in [
        harness.grant(&admin.token, moderator.id).await?,
        harness.revoke(&admin.token, user.id).await?,
        harness.get_json(READERS).await?,
    ] {
        assert_eq!(
            (status, &body["error_code"]),
            (StatusCode::INTERNAL_SERVER_ERROR, &Value::from(120))
        );
    }
    // Reading still follows the list as it stood before the failures.
    let (status, _) = harness.get_as(&user.token, SEARCH).await?;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = harness.get_as(&moderator.token, SEARCH).await?;
    assert_eq!(status, StatusCode::FORBIDDEN);
    Ok(())
}
