//! Discover indexed Git links explicitly; ordinary directories never become trusted repositories.

use std::{
    collections::BTreeSet,
    ffi::OsStr,
    fs,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};

use super::source_inventory::{
    git_paths, validate_relative_path, verify_runtime_credentials_are_private,
};
use crate::{TaskError, TaskResult};

const MAX_SUBMODULE_DEPTH: usize = 16;
const MAX_REPOSITORIES: usize = 256;
const MAX_INVENTORY_FILES: usize = 1_000_000;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Inventory {
    pub(super) files: Vec<PathBuf>,
    pub(super) repositories: Vec<PathBuf>,
}

pub(super) fn discover(root: &Path) -> TaskResult<Inventory> {
    let mut inventory = Inventory {
        files: Vec::new(),
        repositories: Vec::new(),
    };
    visit(root, Path::new(""), 0, &mut inventory)?;
    inventory.files.sort();
    inventory.files.dedup();
    Ok(inventory)
}

fn visit(root: &Path, prefix: &Path, depth: usize, inventory: &mut Inventory) -> TaskResult<()> {
    if depth > MAX_SUBMODULE_DEPTH || inventory.repositories.len() >= MAX_REPOSITORIES {
        return Err(TaskError(
            "public source exceeds the submodule traversal bound".to_owned(),
        ));
    }
    let repository = root.join(prefix);
    verify_runtime_credentials_are_private(&repository)?;
    inventory.repositories.push(repository.clone());
    let links = gitlinks(&repository)?;
    let paths = git_paths(
        &repository,
        &[
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ],
        "list public source files",
    )?;
    for encoded in paths {
        let relative = PathBuf::from(OsStr::from_bytes(&encoded));
        validate_relative_path(&relative)?;
        let path = prefix.join(&relative);
        reject_symlink_parents(root, &path)?;
        if links.contains(&relative) {
            require_submodule(&root.join(&path))?;
            visit(root, &path, depth + 1, inventory)?;
        } else {
            if inventory.files.len() >= MAX_INVENTORY_FILES {
                return Err(TaskError(
                    "public source exceeds the file inventory bound".to_owned(),
                ));
            }
            inventory.files.push(path);
        }
    }
    Ok(())
}

fn gitlinks(repository: &Path) -> TaskResult<BTreeSet<PathBuf>> {
    let entries = git_paths(
        repository,
        &["ls-files", "--stage", "-z"],
        "list indexed Git links",
    )?;
    let mut links = BTreeSet::new();
    for entry in entries {
        let Some(separator) = entry.iter().position(|byte| *byte == b'\t') else {
            return Err(TaskError("Git returned an invalid index entry".to_owned()));
        };
        let fields = entry[..separator]
            .split(|byte| *byte == b' ')
            .collect::<Vec<_>>();
        if fields.len() != 3 || fields[2] != b"0" {
            return Err(TaskError(
                "secret scanning requires a resolved Git index".to_owned(),
            ));
        }
        if fields[0] == b"160000" {
            let path = PathBuf::from(OsStr::from_bytes(&entry[separator + 1..]));
            validate_relative_path(&path)?;
            links.insert(path);
        }
    }
    Ok(links)
}

fn require_submodule(path: &Path) -> TaskResult<()> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        TaskError(format!(
            "submodule is unavailable; run git submodule update --init --recursive: {error}"
        ))
    })?;
    if !metadata.is_dir() {
        return Err(TaskError(
            "indexed submodule is not a real directory".to_owned(),
        ));
    }
    // Without a checkout marker Git would silently discover the parent repository instead.
    if !path.join(".git").exists() {
        return Err(TaskError(
            "submodule is uninitialized; run git submodule update --init --recursive".to_owned(),
        ));
    }
    let canonical = fs::canonicalize(path)
        .map_err(|error| TaskError(format!("failed to resolve submodule checkout: {error}")))?;
    let output = git_paths(
        path,
        &["rev-parse", "--show-toplevel"],
        "verify submodule checkout",
    )?;
    let mut expected = canonical.as_os_str().as_bytes().to_vec();
    expected.push(b'\n');
    if output != [expected] {
        return Err(TaskError(
            "indexed submodule does not identify a repository root".to_owned(),
        ));
    }
    Ok(())
}

fn reject_symlink_parents(root: &Path, relative: &Path) -> TaskResult<()> {
    let Some(parent) = relative.parent() else {
        return Ok(());
    };
    let mut ancestor = root.to_path_buf();
    for component in parent.components() {
        ancestor.push(component);
        let metadata = fs::symlink_metadata(&ancestor)
            .map_err(|error| TaskError(format!("failed to inspect source parent: {error}")))?;
        if !metadata.is_dir() {
            return Err(TaskError(
                "public source parent is not a real directory".to_owned(),
            ));
        }
    }
    Ok(())
}
