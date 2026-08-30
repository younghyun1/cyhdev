//! Fail-closed inventory and snapshot of files eligible for public Git inclusion.

use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{Read, Write},
    os::unix::ffi::OsStrExt,
    path::{Component, Path, PathBuf},
    process::Command,
};

use crate::{TaskError, TaskResult};
use super::credential_policy::{RUNTIME_CREDENTIAL_PATHS, is_runtime_credential_path};

const MAX_SNAPSHOT_FILES: usize = 1_000_000;
const MAX_SNAPSHOT_BYTES: u64 = 1024 * 1024 * 1024;
const MAX_PUBLIC_FILE_BYTES: u64 = 128 * 1024 * 1024;

pub(super) fn copy_public_source(root: &Path, snapshot: &Path) -> TaskResult<()> {
    verify_runtime_credentials_are_private(root)?;
    let paths = git_paths(
        root,
        &["ls-files", "-z", "--cached", "--others", "--exclude-standard"],
        "list public source files",
    )?;
    let mut copied_files = 0usize;
    let mut copied_bytes = 0u64;
    for encoded_path in &paths {
        copied_files = copied_files.saturating_add(1);
        if copied_files > MAX_SNAPSHOT_FILES {
            return Err(TaskError(format!(
                "public source exceeds the {MAX_SNAPSHOT_FILES}-file scan bound"
            )));
        }
        let relative = PathBuf::from(OsStr::from_bytes(encoded_path));
        validate_relative_path(&relative)?;
        if is_runtime_credential_path(&relative) {
            return Err(TaskError(
                "a runtime credential-shaped file is eligible for public Git inclusion"
                    .to_owned(),
            ));
        }
        let source = root.join(&relative);
        let metadata = fs::symlink_metadata(&source).map_err(|error| {
            TaskError(format!(
                "failed to inspect public source candidate {}: {error}",
                relative.display()
            ))
        })?;
        if !metadata.is_file() && !metadata.file_type().is_symlink() {
            return Err(TaskError(format!(
                "public source candidate has unsupported file type: {}",
                relative.display()
            )));
        }
        if metadata.is_file() && metadata.len() > MAX_PUBLIC_FILE_BYTES {
            return Err(TaskError(format!(
                "public source candidate {} exceeds the {MAX_PUBLIC_FILE_BYTES}-byte per-file scan bound",
                relative.display()
            )));
        }
        let destination = snapshot.join(&relative);
        let parent = destination.parent().ok_or_else(|| {
            TaskError(format!("snapshot path has no parent: {}", destination.display()))
        })?;
        fs::create_dir_all(parent).map_err(|error| {
            TaskError(format!("failed to create {}: {error}", parent.display()))
        })?;
        let copied = if metadata.file_type().is_symlink() {
            snapshot_symlink(&source, &destination)?
        } else {
            let remaining = MAX_SNAPSHOT_BYTES.saturating_sub(copied_bytes);
            copy_regular_file_bounded(
                &source,
                &destination,
                remaining.min(MAX_PUBLIC_FILE_BYTES),
            )?
        };
        copied_bytes = add_snapshot_bytes(copied_bytes, copied)?;
    }
    let after = git_paths(
        root,
        &["ls-files", "-z", "--cached", "--others", "--exclude-standard"],
        "relist public source files",
    )?;
    if paths != after {
        return Err(TaskError(
            "public source inventory changed while the secret snapshot was created".to_owned(),
        ));
    }
    verify_runtime_credentials_are_private(root)?;
    Ok(())
}

fn verify_runtime_credentials_are_private(root: &Path) -> TaskResult<()> {
    let tracked = credential_paths(root, false)?;
    let tracked_credentials = tracked
        .iter()
        .map(|path| PathBuf::from(OsStr::from_bytes(path)))
        .filter(|path| is_runtime_credential_path(path))
        .count();
    if tracked_credentials != 0 {
        return Err(TaskError(format!(
            "{tracked_credentials} runtime credential-shaped file(s) are tracked"
        )));
    }

    let ignored = credential_paths(root, true)?;
    for encoded_path in ignored {
        let path = PathBuf::from(OsStr::from_bytes(&encoded_path));
        validate_relative_path(&path)?;
        if is_runtime_credential_path(&path) {
            let metadata = fs::symlink_metadata(root.join(&path)).map_err(|error| {
                TaskError(format!("failed to inspect ignored credential file: {error}"))
            })?;
            if !metadata.is_file() && !metadata.file_type().is_symlink() {
                return Err(TaskError(
                    "ignored runtime credential candidate is not a file".to_owned(),
                ));
            }
        }
    }
    Ok(())
}

fn credential_paths(root: &Path, ignored: bool) -> TaskResult<Vec<Vec<u8>>> {
    let mut arguments = vec!["ls-files", "-z"];
    if ignored {
        arguments.extend(["--others", "--ignored", "--exclude-standard"]);
    } else {
        arguments.push("--cached");
    }
    arguments.push("--");
    arguments.extend(RUNTIME_CREDENTIAL_PATHS);
    git_paths(root, &arguments, "list runtime credential paths")
}

fn git_paths(root: &Path, arguments: &[&str], operation: &str) -> TaskResult<Vec<Vec<u8>>> {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(root)
        .output()
        .map_err(|error| TaskError(format!("failed to {operation}: {error}")))?;
    if !output.status.success() {
        return Err(TaskError(format!(
            "git command to {operation} exited with status {}",
            output.status
        )));
    }
    Ok(output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .map(<[u8]>::to_vec)
        .collect())
}

fn snapshot_symlink(source: &Path, destination: &Path) -> TaskResult<u64> {
    let target = fs::read_link(source)
        .map_err(|error| TaskError(format!("failed to read symlink: {error}")))?;
    let bytes = target.as_os_str().as_bytes();
    fs::write(destination, bytes)
        .map_err(|error| TaskError(format!("failed to snapshot symlink: {error}")))?;
    u64::try_from(bytes.len())
        .map_err(|_source| TaskError("symlink target length does not fit u64".to_owned()))
}

fn copy_regular_file_bounded(
    source: &Path,
    destination: &Path,
    maximum_bytes: u64,
) -> TaskResult<u64> {
    let mut input = File::open(source)
        .map_err(|error| TaskError(format!("failed to open public source: {error}")))?;
    let mut output = File::create(destination)
        .map_err(|error| TaskError(format!("failed to create source snapshot: {error}")))?;
    let mut copied = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let remaining = maximum_bytes.saturating_sub(copied);
        if remaining == 0 {
            let mut overflow = [0u8; 1];
            if input
                .read(&mut overflow)
                .map_err(|error| TaskError(format!("failed to read public source: {error}")))?
                == 0
            {
                return Ok(copied);
            }
            return Err(snapshot_size_error());
        }
        let read_limit = match usize::try_from(remaining) {
            Ok(remaining) => remaining.min(buffer.len()),
            Err(_source) => buffer.len(),
        };
        let count = input
            .read(&mut buffer[..read_limit])
            .map_err(|error| TaskError(format!("failed to read public source: {error}")))?;
        if count == 0 {
            return Ok(copied);
        }
        output
            .write_all(&buffer[..count])
            .map_err(|error| TaskError(format!("failed to write source snapshot: {error}")))?;
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
        .ok_or_else(|| TaskError("public source byte count overflowed u64".to_owned()))?;
    if total > MAX_SNAPSHOT_BYTES {
        Err(snapshot_size_error())
    } else {
        Ok(total)
    }
}

fn snapshot_size_error() -> TaskError {
    TaskError(format!(
        "public source exceeds the {MAX_SNAPSHOT_BYTES}-byte scan bound"
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
        Err(TaskError("git returned an unsafe source path".to_owned()))
    }
}
