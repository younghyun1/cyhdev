//! Redacted secret scans over current public source and complete Git history.

mod credential_policy;
mod git_inventory;
mod source_inventory;
#[cfg(test)]
mod tests;

use std::{fs, path::Path, process::Command};

use crate::{TaskError, TaskResult, run_command};

pub(crate) fn run(root: &Path) -> TaskResult<()> {
    let report_dir = root.join("target/secret-scan");
    fs::create_dir_all(&report_dir).map_err(|error| {
        TaskError(format!(
            "failed to create {}: {error}",
            report_dir.display()
        ))
    })?;
    let snapshot = report_dir.join(format!("current-tree-{}", std::process::id()));
    if snapshot.exists() {
        return Err(TaskError(format!(
            "secret-scan snapshot already exists: {}",
            snapshot.display()
        )));
    }
    fs::create_dir(&snapshot).map_err(|error| {
        TaskError(format!(
            "failed to create secret-scan snapshot {}: {error}",
            snapshot.display()
        ))
    })?;

    let repositories = match source_inventory::copy_public_source(root, &snapshot) {
        Ok(repositories) => repositories,
        Err(error) => {
            let _cleanup_result = fs::remove_dir_all(&snapshot);
            return Err(error);
        }
    };

    let current = run_gitleaks(
        root,
        "dir",
        &snapshot,
        &report_dir.join("current.json"),
        None,
    );
    let cleanup = fs::remove_dir_all(&snapshot).map_err(|error| {
        TaskError(format!(
            "failed to remove secret-scan snapshot {}: {error}",
            snapshot.display()
        ))
    });
    let history = finish(repositories.iter().enumerate().map(|(index, repository)| {
        let report = if index == 0 {
            report_dir.join("history.json")
        } else {
            report_dir.join(format!("submodule-history-{index}.json"))
        };
        (
            "repository history",
            run_gitleaks(root, "git", repository, &report, Some("--all")),
        )
    }));

    finish([
        ("current tree", current),
        ("snapshot cleanup", cleanup),
        ("Git history", history),
    ])
}

fn run_gitleaks(
    root: &Path,
    mode: &str,
    target: &Path,
    report_path: &Path,
    log_options: Option<&str>,
) -> TaskResult<()> {
    let mut command = Command::new("gitleaks");
    command.args([
        mode,
        "--redact=100",
        "--no-banner",
        "--no-color",
        "--log-level=warn",
        "--timeout=600",
        "--report-format=json",
        "--report-path",
    ]);
    command.arg(report_path);
    if let Some(options) = log_options {
        command.arg(format!("--log-opts={options}"));
    }
    command.arg("--config").arg(root.join(".gitleaks.toml"));
    command.arg(target).current_dir(root);
    run_command(&mut command)
}

fn finish(results: impl IntoIterator<Item = (&'static str, TaskResult<()>)>) -> TaskResult<()> {
    let failures = results
        .into_iter()
        .filter_map(|(name, result)| match result {
            Ok(()) => None,
            Err(error) => Some(format!("{name}: {error}")),
        })
        .collect::<Vec<_>>();
    if failures.is_empty() {
        Ok(())
    } else {
        Err(TaskError(format!(
            "secret scan failed:\n  {}",
            failures.join("\n  ")
        )))
    }
}
