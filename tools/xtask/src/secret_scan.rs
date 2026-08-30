//! Redacted secret scans over current source and complete Git history.

use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{Read, Write},
    os::unix::ffi::OsStrExt,
    path::{Component, Path, PathBuf},
    process::Command,
};

use crate::{TaskError, TaskResult, run_command};

const MAX_SNAPSHOT_FILES: usize = 1_000_000;
const MAX_SNAPSHOT_BYTES: u64 = 1024 * 1024 * 1024;

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

    if let Err(error) = copy_current_source(root, &snapshot) {
        let _cleanup_result = fs::remove_dir_all(&snapshot);
        return Err(error);
    }

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
    let history = run_gitleaks(
        root,
        "git",
        Path::new("."),
        &report_dir.join("history.json"),
        Some("--all"),
    );

    finish([
        ("current tree", current),
        ("snapshot cleanup", cleanup),
        ("Git history", history),
    ])
}

fn copy_current_source(root: &Path, snapshot: &Path) -> TaskResult<()> {
    let output = Command::new("git")
        .args(["ls-files", "-z", "--cached", "--others", "--exclude-standard"])
        .current_dir(root)
        .output()
        .map_err(|error| TaskError(format!("failed to list current source files: {error}")))?;
    if !output.status.success() {
        return Err(TaskError(format!(
            "git ls-files exited with status {}",
            output.status
        )));
    }

    let mut copied_files = 0usize;
    let mut copied_bytes = 0u64;
    for encoded_path in output.stdout.split(|byte| *byte == 0).filter(|path| !path.is_empty()) {
        copied_files = copied_files.saturating_add(1);
        if copied_files > MAX_SNAPSHOT_FILES {
            return Err(TaskError(format!(
                "current source exceeds the {MAX_SNAPSHOT_FILES}-file scan bound"
            )));
        }
        let relative = PathBuf::from(OsStr::from_bytes(encoded_path));
        validate_relative_path(&relative)?;
        let source = root.join(&relative);
        let metadata = match fs::symlink_metadata(&source) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(TaskError(format!(
                    "failed to inspect {}: {error}",
                    source.display()
                )));
            }
        };
        if !metadata.is_file() && !metadata.file_type().is_symlink() {
            continue;
        }
        let destination = snapshot.join(&relative);
        let parent = destination.parent().ok_or_else(|| {
            TaskError(format!("snapshot path has no parent: {}", destination.display()))
        })?;
        fs::create_dir_all(parent).map_err(|error| {
            TaskError(format!("failed to create {}: {error}", parent.display()))
        })?;
        if metadata.file_type().is_symlink() {
            let target = fs::read_link(&source).map_err(|error| {
                TaskError(format!("failed to read symlink {}: {error}", source.display()))
            })?;
            let target_bytes = target.as_os_str().as_bytes();
            let target_length = u64::try_from(target_bytes.len()).map_err(|_source| {
                TaskError(format!(
                    "symlink target length does not fit u64: {}",
                    source.display()
                ))
            })?;
            copied_bytes = add_snapshot_bytes(copied_bytes, target_length)?;
            fs::write(&destination, target.as_os_str().as_bytes()).map_err(|error| {
                TaskError(format!("failed to snapshot {}: {error}", source.display()))
            })?;
        } else {
            let remaining = MAX_SNAPSHOT_BYTES.saturating_sub(copied_bytes);
            let copied = copy_regular_file_bounded(&source, &destination, remaining)?;
            copied_bytes = add_snapshot_bytes(copied_bytes, copied)?;
        }
    }
    Ok(())
}

fn copy_regular_file_bounded(source: &Path, destination: &Path, maximum_bytes: u64) -> TaskResult<u64> {
    let mut input = File::open(source).map_err(|error| {
        TaskError(format!("failed to open {} for snapshot: {error}", source.display()))
    })?;
    let mut output = File::create(destination).map_err(|error| {
        TaskError(format!(
            "failed to create snapshot file {}: {error}",
            destination.display()
        ))
    })?;
    let mut copied = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let remaining = maximum_bytes.saturating_sub(copied);
        if remaining == 0 {
            let mut overflow = [0_u8; 1];
            let count = input.read(&mut overflow).map_err(|error| {
                TaskError(format!("failed to read {}: {error}", source.display()))
            })?;
            if count == 0 {
                return Ok(copied);
            }
            return Err(snapshot_size_error());
        }
        let read_limit = match usize::try_from(remaining) {
            Ok(remaining) => remaining.min(buffer.len()),
            Err(_source) => buffer.len(),
        };
        let count = input.read(&mut buffer[..read_limit]).map_err(|error| {
            TaskError(format!("failed to read {}: {error}", source.display()))
        })?;
        if count == 0 {
            return Ok(copied);
        }
        output.write_all(&buffer[..count]).map_err(|error| {
            TaskError(format!(
                "failed to write snapshot file {}: {error}",
                destination.display()
            ))
        })?;
        let count = u64::try_from(count)
            .map_err(|_source| TaskError("snapshot read length does not fit u64".to_owned()))?;
        copied = copied
            .checked_add(count)
            .ok_or_else(|| TaskError("snapshot byte count overflowed u64".to_owned()))?;
    }
}

fn add_snapshot_bytes(current: u64, added: u64) -> TaskResult<u64> {
    let total = current
        .checked_add(added)
        .ok_or_else(|| TaskError("current source byte count overflowed u64".to_owned()))?;
    if total > MAX_SNAPSHOT_BYTES {
        Err(snapshot_size_error())
    } else {
        Ok(total)
    }
}

fn snapshot_size_error() -> TaskError {
    TaskError(format!(
        "current source exceeds the {MAX_SNAPSHOT_BYTES}-byte scan bound"
    ))
}

fn validate_relative_path(path: &Path) -> TaskResult<()> {
    let valid = !path.as_os_str().is_empty()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)));
    if valid {
        Ok(())
    } else {
        Err(TaskError(format!(
            "git returned an unsafe source path: {}",
            path.display()
        )))
    }
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
        "--max-target-megabytes=64",
        "--timeout=600",
        "--report-format=json",
        "--report-path",
    ]);
    command.arg(report_path);
    if let Some(options) = log_options {
        command.arg(format!("--log-opts={options}"));
    }
    command.arg(target).current_dir(root);
    run_command(&mut command)
}

fn finish<const N: usize>(results: [(&'static str, TaskResult<()>); N]) -> TaskResult<()> {
    let failures: Vec<String> = results
        .into_iter()
        .filter_map(|(name, result)| match result {
            Ok(()) => None,
            Err(error) => Some(format!("{name}: {error}")),
        })
        .collect();
    if failures.is_empty() {
        Ok(())
    } else {
        Err(TaskError(format!(
            "secret scan failed:\n  {}",
            failures.join("\n  ")
        )))
    }
}
