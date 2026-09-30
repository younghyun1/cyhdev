//! Complete successful coverage is required independently of negative probes.

use std::{collections::BTreeSet, path::Path};

use serde::{Deserialize, Serialize};

use crate::{TaskError, TaskResult};

use super::files;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Surface {
    pub schema_version: u8,
    pub operations: BTreeSet<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Policy {
    pub schema_version: u8,
    pub pages: BTreeSet<String>,
    pub scenarios: BTreeSet<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Report {
    pub schema_version: u8,
    pub stage: String,
    pub binary_sha256: String,
    pub campaign_sha256: String,
    pub operations: BTreeSet<String>,
    pub pages: BTreeSet<String>,
    pub scenarios: BTreeSet<String>,
    pub failures: u64,
}

pub(super) fn check(
    report: &Report,
    surface: &Surface,
    policy: &Policy,
    stage: &str,
    binary_digest: &str,
    campaign_digest: &str,
) -> TaskResult<()> {
    if report.schema_version != 1
        || surface.schema_version != 1
        || policy.schema_version != 1
        || report.stage != stage
        || report.binary_sha256 != binary_digest
        || report.campaign_sha256 != campaign_digest
        || report.failures != 0
    {
        return Err(TaskError(
            "coverage identity mismatch or failed campaign".into(),
        ));
    }
    if surface.operations.is_empty() || policy.pages.is_empty() || policy.scenarios.is_empty() {
        return Err(TaskError("coverage requirements cannot be empty".into()));
    }
    let missing = [
        (
            "operations",
            surface
                .operations
                .difference(&report.operations)
                .cloned()
                .collect::<Vec<_>>(),
        ),
        (
            "pages",
            policy.pages.difference(&report.pages).cloned().collect(),
        ),
        (
            "scenarios",
            policy
                .scenarios
                .difference(&report.scenarios)
                .cloned()
                .collect(),
        ),
    ];
    let missing = missing
        .into_iter()
        .filter(|(_, values)| !values.is_empty())
        .map(|(kind, values)| format!("{kind}: {}", values.join(", ")))
        .collect::<Vec<_>>();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(TaskError(format!(
            "incomplete successful coverage; {}",
            missing.join("; ")
        )))
    }
}

pub(super) fn verify(
    root: &Path,
    run: &Path,
    stage: &str,
    binary: &Path,
    campaign: &Path,
) -> TaskResult<()> {
    let report = files::read_json(&run.join(format!("{stage}-coverage.json")), 8 * 1024 * 1024)?;
    let surface = files::read_json(&run.join("surface.json"), 1024 * 1024)?;
    let policy = files::read_json(&root.join("tools/optimization/coverage.json"), 1024 * 1024)?;
    check(
        &report,
        &surface,
        &policy,
        stage,
        &files::digest(binary)?,
        &files::digest(campaign)?,
    )
}
