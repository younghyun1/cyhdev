//! Synthetic workload contracts; no fixture services or files are accessed.
use super::{benchmark_cases, workflow_cases};
use axum::response::IntoResponse;
use serde_json::{Value, json};
use std::collections::BTreeSet;

#[test]
fn capacity_mix_retains_ten_distinct_equal_weight_public_reads() {
    let cases = benchmark_cases();
    let expected = BTreeSet::from([
        "/api/blog/posts",
        "/api/blog/posts/${post_id}",
        "/api/blog/search?q=fixture",
        "/api/forum/topics",
        "/api/photographs/get",
        "/api/dropdown/country",
        "/api/dropdown/language",
        "/api/i18n/ui-text?locale=en-US",
        "/api/healthcheck/server",
        "/api/visitor-board",
    ]);
    let paths: BTreeSet<_> = cases
        .iter()
        .filter_map(|case| case["path"].as_str())
        .collect();
    assert_eq!(cases.len(), 10);
    assert_eq!(paths, expected);
    for case in &cases {
        assert_eq!(case["kind"], "request");
        assert_eq!(case["method"], "GET");
        assert_eq!(case["status"], 200);
        assert_ne!(case["route"], "/api/healthcheck/state");
    }
}

#[test]
fn health_capacity_case_asserts_raw_json_instead_of_an_envelope() {
    let cases = benchmark_cases();
    let health: Vec<_> = cases
        .iter()
        .filter(|case| case["route"] == "/api/healthcheck/server")
        .collect();
    assert_eq!(health.len(), 1);
    assert_eq!(health[0]["content_type"], "application/json");
    assert_eq!(health[0]["minimum_bytes"], 16);
    assert_eq!(health[0].get("json_pointer"), None);
    assert_eq!(health[0].get("equals"), None);
    for case in cases
        .iter()
        .filter(|case| case["route"] != "/api/healthcheck/server")
    {
        assert_eq!(case["json_pointer"], "/success");
        assert_eq!(case["equals"], true);
    }
}

#[tokio::test]
async fn raw_server_health_response_satisfies_its_capacity_assertions() -> anyhow::Result<()> {
    let cases = benchmark_cases();
    let health = match cases
        .iter()
        .find(|case| case["route"] == "/api/healthcheck/server")
    {
        Some(health) => health,
        None => anyhow::bail!("capacity health case missing"),
    };
    let response = rust_be_template::features::server_status::api::http::healthcheck()
        .await
        .into_response();
    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(response.headers()["content-type"], "application/json");
    let body = match axum::body::to_bytes(response.into_body(), 16 * 1024).await {
        Ok(body) => body,
        Err(error) => return Err(error.into()),
    };
    let minimum_bytes = match health["minimum_bytes"].as_u64() {
        Some(minimum_bytes) => minimum_bytes,
        None => anyhow::bail!("capacity health body assertion missing"),
    };
    assert!(body.len() as u64 >= minimum_bytes);
    let metadata: Value = match serde_json::from_slice(&body) {
        Ok(metadata) => metadata,
        Err(error) => return Err(error.into()),
    };
    for field in ["build_time", "axum_version", "rust_version"] {
        assert!(metadata[field].is_string());
    }
    assert_eq!(metadata.get("success"), None);
    Ok(())
}

#[test]
fn fresh_database_probe_keeps_its_functional_host_scenario_coverage() -> anyhow::Result<()> {
    let scenarios = [json!("host.health-stats-websocket")];
    let operations = [
        json!("GET /api/healthcheck/server"),
        json!("GET /api/healthcheck/state"),
        json!("GET /api/healthcheck/fastfetch"),
    ];
    let workflows = workflow_cases(&scenarios, &operations)?;
    assert_eq!(workflows.len(), 1);
    assert_eq!(workflows[0]["actor"], "anonymous");
    assert_eq!(workflows[0]["steps"][0]["kind"], "fixture_scenario");
    assert_eq!(workflows[0]["steps"][0]["name"], scenarios[0]);
    assert_eq!(
        workflows[0]["steps"][0]["operations"],
        Value::from(operations.to_vec())
    );
    Ok(())
}
