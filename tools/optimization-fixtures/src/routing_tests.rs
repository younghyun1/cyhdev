//! Scenario ownership and page assertions for the archive, plus coverage-policy agreement.

use super::{actor, owner, page_assertion, page_path};
use serde_json::Value;

const SCENARIO: &str = "kcna.search-read-trends-reload";
const PAGES: [&str; 5] = [
    "/projects/kcna-archive",
    "/projects/kcna-archive/documents/:documentId",
    "/projects/kcna-archive/trends",
    "/projects/kcna-archive/status",
    "/admin/kcna-archive",
];

fn policy() -> anyhow::Result<Value> {
    Ok(serde_json::from_str(include_str!(
        "../../optimization/coverage.json"
    ))?)
}

#[test]
fn every_archive_operation_belongs_to_the_archive_scenario() {
    for operation in [
        "GET /api/kcna-archive/search",
        "GET /api/kcna-archive/histogram",
        "GET /api/kcna-archive/trends",
        "GET /api/kcna-archive/documents/{document_id}",
        "GET /api/kcna-archive/export",
        "GET /api/kcna-archive/access",
        "GET /api/admin/kcna-archive/status",
        "POST /api/admin/kcna-archive/reload",
        "GET /api/admin/kcna-archive/readers",
        "POST /api/admin/kcna-archive/readers",
        "DELETE /api/admin/kcna-archive/readers/{user_id}",
    ] {
        assert_eq!(owner(operation), SCENARIO, "{operation}");
    }
    assert_eq!(actor(SCENARIO), "admin");
    // Neighbouring admin families keep their own scenarios.
    assert_eq!(
        owner("GET /api/admin/authorization/roles"),
        "authorization.roles-permissions-audit"
    );
}

/// The generated workload declares each operation under its owner scenario, so an
/// operation owned by a scenario that coverage.json does not list would never
/// be requested.
#[test]
fn every_openapi_operation_is_owned_by_a_listed_scenario() -> anyhow::Result<()> {
    use utoipa::OpenApi;
    let policy = policy()?;
    let scenarios: Vec<&str> = policy["scenarios"]
        .as_array()
        .map(|items| items.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let spec = serde_json::to_value(rust_be_template::docs::ApiDoc::openapi())?;
    let paths = spec["paths"]
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("OpenAPI has no paths"))?;
    let mut archive = 0;
    for (path, item) in paths {
        for method in ["get", "post", "put", "patch", "delete"] {
            if item.get(method).is_none() {
                continue;
            }
            let operation = format!("{} {path}", method.to_ascii_uppercase());
            let scenario = owner(&operation);
            assert!(scenarios.contains(&scenario), "{operation} -> {scenario}");
            archive += usize::from(scenario == SCENARIO);
        }
    }
    assert_eq!(archive, 11);
    Ok(())
}

#[test]
fn archive_pages_are_listed_and_asserted_for_the_superuser() -> anyhow::Result<()> {
    let policy = policy()?;
    let pages: Vec<&str> = policy["pages"]
        .as_array()
        .map(|pages| pages.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let scenarios: Vec<&str> = policy["scenarios"]
        .as_array()
        .map(|items| items.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    assert!(scenarios.contains(&SCENARIO));
    let mut selectors = std::collections::BTreeSet::new();
    for page in PAGES {
        assert!(pages.contains(&page), "{page} missing from coverage.json");
        let (selector, actor) = page_assertion(page);
        assert_eq!(actor, "admin");
        assert!(selector.starts_with(".kcna-"), "{page}: {selector}");
        selectors.insert(selector);
    }
    assert_eq!(
        selectors.len(),
        PAGES.len(),
        "selectors must differ per page"
    );
    Ok(())
}

#[test]
fn only_the_trends_page_needs_a_query_string() {
    assert_eq!(
        page_path("/projects/kcna-archive/trends"),
        Some("/projects/kcna-archive/trends?terms=lantern,harbor")
    );
    for page in [
        "/projects/kcna-archive",
        "/projects/kcna-archive/documents/:documentId",
        "/projects/kcna-archive/status",
        "/admin/kcna-archive",
    ] {
        assert_eq!(page_path(page), None);
    }
    assert_eq!(page_path("*404"), Some("/optimization-missing-page"));
}
