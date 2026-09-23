//! Development staging for the vendored EU5 Slint browser application.

use std::{fs, path::Path, process::Command};

use crate::{TaskError, TaskResult, run_command};

const EU5_SUBMODULE: &str = "vendor/eu5-location-filter";

pub(super) fn run(root: &Path) -> TaskResult<()> {
    require_checkout(root)?;
    let application = root.join("solid-csr-spa-template/public/eu5-locations-db/app");
    if application.exists() {
        fs::remove_dir_all(&application).map_err(|error| {
            TaskError(format!(
                "could not clear staged EU5 browser assets at {}: {error}",
                application.display()
            ))
        })?;
    }
    let package = application.join("pkg");
    fs::create_dir_all(&package).map_err(|error| {
        TaskError(format!(
            "could not create EU5 browser asset directory {}: {error}",
            package.display()
        ))
    })?;
    let host_document = root.join("vendor/eu5-location-filter/web/index.html");
    fs::copy(&host_document, application.join("index.html")).map_err(|error| {
        TaskError(format!(
            "could not stage EU5 host document {}: {error}",
            host_document.display()
        ))
    })?;

    let mut command = command(root, &package);
    run_command(&mut command)
}

pub(crate) fn require_checkout(root: &Path) -> TaskResult<()> {
    let source = root.join(EU5_SUBMODULE);
    for relative in [
        "Cargo.toml",
        "Cargo.lock",
        "web/index.html",
        "assets/eu5-locations.bitcode.zst",
        "assets/eu5-indexes.bitcode.zst",
    ] {
        let required = source.join(relative);
        if !required.is_file() {
            return Err(TaskError(format!(
                "EU5 source checkout is incomplete at {}; run `git submodule update --init --recursive` from {}",
                required.display(),
                root.display()
            )));
        }
    }
    Ok(())
}

/// Requires the recorded submodule revisions, unmodified, before an optimized
/// build. Docker copies the submodule working tree, so a moved, conflicted, or
/// locally edited checkout would ship source that no commit records.
/// Development staging deliberately builds local edits and does not call this.
pub(crate) fn require_pinned_checkout(root: &Path) -> TaskResult<()> {
    require_checkout(root)?;
    let status = git_stdout(
        root,
        &["submodule", "status", "--recursive"],
        "list submodule revisions",
    )?;
    check_submodule_status(&status)?;
    let changes = git_stdout(
        &root.join(EU5_SUBMODULE),
        &["status", "--porcelain"],
        "list EU5 submodule working tree changes",
    )?;
    check_submodule_worktree(&changes)
}

/// Rejects any `git submodule status` line whose marker is not a space:
/// `-` uninitialized, `+` checked out away from the gitlink, `U` conflicted.
pub(crate) fn check_submodule_status(output: &str) -> TaskResult<()> {
    let problems = output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| {
            let mut characters = line.chars();
            let marker = characters.next();
            let path = characters
                .as_str()
                .split_whitespace()
                .nth(1)
                .unwrap_or("<unknown path>");
            match marker {
                Some(' ') => None,
                Some('-') => Some(format!(
                    "{path} is not initialized; run `git submodule update --init --recursive`"
                )),
                Some('+') => Some(format!(
                    "{path} is checked out at a commit other than the recorded gitlink; run `git submodule update --init --recursive` to restore it, or commit the gitlink change first"
                )),
                Some('U') => Some(format!(
                    "{path} has merge conflicts; resolve them and commit the gitlink first"
                )),
                _ => Some(format!("unrecognized `git submodule status` line: {line}")),
            }
        })
        .collect::<Vec<_>>();
    if problems.is_empty() {
        Ok(())
    } else {
        Err(TaskError(format!(
            "optimized builds require pinned, unmodified submodules:\n  {}",
            problems.join("\n  ")
        )))
    }
}

/// Rejects tracked edits and untracked, unignored files in the EU5 checkout;
/// both would enter the Docker build context.
pub(crate) fn check_submodule_worktree(porcelain: &str) -> TaskResult<()> {
    let changed = porcelain
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();
    if changed == 0 {
        Ok(())
    } else {
        Err(TaskError(format!(
            "{EU5_SUBMODULE} has {changed} uncommitted or untracked path(s); inspect them with `git -C {EU5_SUBMODULE} status`, then either commit them upstream and advance the gitlink, or move them aside to restore the pinned checkout"
        )))
    }
}

fn git_stdout(directory: &Path, arguments: &[&str], operation: &str) -> TaskResult<String> {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(directory)
        .output()
        .map_err(|error| TaskError(format!("failed to {operation}: {error}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(TaskError(format!(
            "git could not {operation} in {}: {}",
            directory.display(),
            stderr.trim()
        )));
    }
    String::from_utf8(output.stdout).map_err(|error| {
        TaskError(format!(
            "git returned non-UTF-8 output while trying to {operation}: {error}"
        ))
    })
}

pub(super) fn command(root: &Path, output: &Path) -> Command {
    let mut command = Command::new("wasm-pack");
    command
        .arg("build")
        .arg(root.join("vendor/eu5-location-filter"))
        .args([
            "--dev",
            "--target",
            "web",
            "--no-pack",
            "--no-typescript",
            "--out-dir",
        ])
        .arg(output)
        .args([
            "--",
            "--locked",
            "--no-default-features",
            "--features",
            "web",
        ])
        .current_dir(root);
    command
}
