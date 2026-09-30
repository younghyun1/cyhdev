//! Source-checked operation inventory and an explicit fixture campaign skeleton.

use super::{
    coverage::{Policy, Report, Surface},
    files,
};
use crate::{TaskError, TaskResult, run_command};
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

/// Generate development-profile metadata without running a server or fixture hook.
pub(super) fn generate(root: &Path, directory: &Path) -> TaskResult<()> {
    if directory.join("campaign.template.json").exists() || directory.join("surface.json").exists()
    {
        return Err(TaskError(
            "inventory output already exists; choose a fresh directory".into(),
        ));
    }
    match fs::create_dir_all(directory) {
        Ok(()) => {}
        Err(error) => {
            return Err(TaskError(format!(
                "cannot create inventory directory: {error}"
            )));
        }
    }
    export(root, &directory.join("surface.json"))?;
    let surface: Surface = files::read_json(&directory.join("surface.json"), 1024 * 1024)?;
    let policy: Policy =
        files::read_json(&root.join("tools/optimization/coverage.json"), 1024 * 1024)?;
    let mut campaign: Value = files::read_json(
        &root.join("tools/optimization/campaign.example.json"),
        1024 * 1024,
    )?;
    let mut steps = Vec::new();
    for operation in &surface.operations {
        let (method, route) = match operation.split_once(' ') {
            Some(parts) => parts,
            None => return Err(TaskError("invalid operation inventory".into())),
        };
        let mut step = json!({"kind":"request","method":method,"route":route,"path":route.replace('{', "${"),"status":200,"json_pointer":"/success","equals":true});
        if matches!(method, "POST" | "PATCH" | "PUT") {
            step["body"] = json!({"REPLACE_WITH_VALID_FIXTURE_REQUEST":true});
        }
        steps.push(step);
    }
    let mut workflows = vec![json!({"name":"api.operation-matrix","actor":"admin","steps":steps})];
    for scenario in &policy.scenarios {
        workflows.push(json!({"name":scenario,"actor":"member","steps":[{"kind":"command","argv":["REPLACE_WITH_DISPOSABLE_RUST_SCENARIO_DRIVER"]}]}));
    }
    campaign["workflows"] = json!(workflows);
    campaign["pages"] = json!(
        policy
            .pages
            .iter()
            .map(|page| (
                page.clone(),
                json!({"selector":"REPLACE_WITH_PAGE_SUCCESS_SELECTOR"})
            ))
            .collect::<serde_json::Map<_, _>>()
    );
    files::write_json(&directory.join("campaign.template.json"), &campaign)?;
    println!(
        "Generated {} HTTP operations, {} page states and {} scenario slots in {}; replace placeholders and verify success semantics",
        surface.operations.len(),
        policy.pages.len(),
        policy.scenarios.len(),
        directory.display()
    );
    Ok(())
}

pub(super) fn export(root: &Path, path: &Path) -> TaskResult<()> {
    run_command(
        Command::new("cargo")
            .args([
                "run",
                "--locked",
                "--package",
                "rust-be-template",
                "--bin",
                "openapi-contracts",
                "--",
                "training-surface",
            ])
            .arg(path)
            .current_dir(root),
    )
}

/// Reject incomplete fixture definitions before optimized compilation.
pub(super) fn preflight(root: &Path, campaign: &Path, surface: &Path) -> TaskResult<()> {
    let campaign: Value = files::read_json(campaign, 1024 * 1024)?;
    let surface: Surface = files::read_json(surface, 1024 * 1024)?;
    let policy: Policy =
        files::read_json(&root.join("tools/optimization/coverage.json"), 1024 * 1024)?;
    if campaign.to_string().contains("REPLACE_") || campaign.get("benchmark").is_none() {
        return Err(TaskError(
            "finish campaign placeholders and benchmark configuration before any optimized build"
                .into(),
        ));
    }
    let workflows = match campaign.get("workflows").and_then(Value::as_array) {
        Some(flows) => flows,
        None => return Err(TaskError("campaign has no workflows".into())),
    };
    let mut operations = std::collections::BTreeSet::new();
    let mut scenarios = std::collections::BTreeSet::new();
    for flow in workflows {
        if let Some(name) = flow.get("name").and_then(Value::as_str) {
            scenarios.insert(name.to_owned());
        }
        if let Some(steps) = flow.get("steps").and_then(Value::as_array) {
            for step in steps {
                if step.get("kind").and_then(Value::as_str) == Some("fixture_scenario")
                    && let Some(declared) = step.get("operations").and_then(Value::as_array)
                {
                    for operation in declared {
                        if let Some(operation) = operation.as_str() {
                            operations.insert(operation.to_owned());
                        }
                    }
                }
                if step.get("kind").and_then(Value::as_str) == Some("request")
                    && step
                        .get("status")
                        .and_then(Value::as_u64)
                        .is_some_and(|status| (200..400).contains(&status))
                    && let (Some(method), Some(route)) = (
                        step.get("method").and_then(Value::as_str),
                        step.get("route").and_then(Value::as_str),
                    )
                {
                    operations.insert(format!("{method} {route}"));
                }
            }
        }
    }
    let pages = match campaign.get("pages").and_then(Value::as_object) {
        Some(pages) => pages.keys().cloned().collect(),
        None => std::collections::BTreeSet::new(),
    };
    let report = Report {
        schema_version: 1,
        stage: "preflight".into(),
        binary_sha256: "preflight".into(),
        campaign_sha256: "preflight".into(),
        operations,
        pages,
        scenarios,
        failures: 0,
    };
    super::coverage::check(
        &report,
        &surface,
        &policy,
        "preflight",
        "preflight",
        "preflight",
    )
}
