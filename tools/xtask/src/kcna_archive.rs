//! Writing the KCNA archive snapshot and copying it to a server.
//!
//! The website serves the archive from a snapshot directory it reads into
//! memory. The snapshot is data, not code: it is built from the collector's
//! PostgreSQL corpus, lives outside Git and outside the backend binary, and is
//! copied to the server on its own, so changing either one never requires
//! rebuilding or reshipping the other.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::{TaskError, TaskResult, run_command};

/// Where the development backend looks for the snapshot, relative to the
/// workspace root. The backend resolves the same path relative to its own
/// working directory.
const LOCAL_SNAPSHOT: &str = "rust-be-template/data/kcna_archive";

/// Brings the local snapshot up to date with the collector's database.
///
/// Runs the collector in the development profile, whose search and
/// compression dependencies are optimized, so a full rebuild of the corpus
/// takes well under a minute. `DATABASE_URL` must name the corpus database.
pub(super) fn run_snapshot(root: &Path, arguments: &[String]) -> TaskResult<()> {
    let mut command = snapshot_command(root, arguments);
    run_command(&mut command)
}

pub(crate) fn snapshot_command(root: &Path, arguments: &[String]) -> Command {
    let mut command = Command::new("cargo");
    command
        .args(["run", "--locked", "--package", "kcna-archive", "--"])
        .arg("--snapshot-dir")
        .arg(local_snapshot(root))
        .args(["database", "snapshot"])
        .args(arguments)
        .current_dir(root);
    command
}

/// Copies the local snapshot to `DESTINATION`, an rsync target such as
/// `host:dist/data/kcna_archive/`.
///
/// Changed files are staged beside their final names and moved into place
/// together at the end of the transfer, so a server that loads the directory
/// meanwhile sees either the old snapshot or the new one. Index segments are
/// immutable and named by content, so an incremental snapshot transfers only
/// its new segments. A running server keeps serving its loaded copy until it
/// restarts or an administrator reloads the snapshot from the status page.
pub(super) fn run_push(root: &Path, arguments: &[String]) -> TaskResult<()> {
    let destination = match arguments {
        [destination] => destination,
        _ => {
            return Err(TaskError(
                "usage: cargo xtask kcna-snapshot-push DESTINATION (an rsync target such as host:dist/data/kcna_archive/)"
                    .to_owned(),
            ));
        }
    };
    validate_destination(destination)?;
    let source = local_snapshot(root);
    if !source.join("snapshot.json").is_file() {
        return Err(TaskError(format!(
            "{} holds no snapshot; run `cargo xtask kcna-snapshot` first",
            source.display()
        )));
    }
    let mut command = push_command(&source, destination);
    run_command(&mut command)
}

pub(crate) fn push_command(source: &Path, destination: &str) -> Command {
    let mut from = source.as_os_str().to_owned();
    // A trailing slash copies the directory's contents, not the directory.
    from.push("/");
    let mut command = Command::new("rsync");
    command
        .args([
            "--archive",
            "--delete-delay",
            "--delay-updates",
            "--human-readable",
            "--info=stats1",
            // Lock files belong to the process that wrote the index.
            "--exclude=.tantivy-*.lock",
        ])
        .arg(from)
        .arg(destination);
    command
}

fn local_snapshot(root: &Path) -> PathBuf {
    root.join(LOCAL_SNAPSHOT)
}

/// Accepts a plain rsync target and nothing rsync would read as an option or
/// that a remote shell could expand.
pub(crate) fn validate_destination(destination: &str) -> TaskResult<()> {
    let allowed = |character: char| {
        character.is_ascii_alphanumeric()
            || matches!(character, '.' | '_' | '-' | '/' | ':' | '@' | '~')
    };
    if destination.is_empty()
        || destination.len() > 256
        || destination.starts_with('-')
        || !destination.chars().all(allowed)
        || !destination.ends_with('/')
    {
        return Err(TaskError(format!(
            "`{destination}` is not an accepted rsync destination: use letters, digits and . _ - / : @ ~ only, and end with a slash"
        )));
    }
    Ok(())
}
