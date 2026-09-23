//! Private per-batch staging for photograph batch uploads.
//!
//! Staged originals still carry full EXIF, including GPS coordinates, so each
//! batch gets its own directory created with mode 0700 under the system
//! temporary directory, and each file is created with mode 0600. Directory
//! names carry a random per-process token, which lets the startup sweep remove
//! every staging directory left by an earlier process without touching ones
//! this process has already created.

use std::{
    fs::Permissions,
    io,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use tempfile::TempDir;
use tokio::io::AsyncWriteExt;
use tracing::warn;
use uuid::Uuid;

/// Shared by current and legacy staging names; the fixed `cyhdev-batch`
/// root used before per-batch directories also matches.
const BATCH_DIR_PREFIX: &str = "cyhdev-batch";

/// Random token identifying directories created by this process.
static PROCESS_TOKEN: LazyLock<String> = LazyLock::new(|| Uuid::new_v4().simple().to_string());

fn current_prefix() -> String {
    format!("{BATCH_DIR_PREFIX}-{}-", *PROCESS_TOKEN)
}

/// One batch's staging directory. Dropping it removes the directory and its
/// files on a blocking thread; [`BatchStagingDir::remove`] awaits removal.
pub struct BatchStagingDir {
    dir: Option<TempDir>,
    path: PathBuf,
}

impl BatchStagingDir {
    pub async fn create() -> io::Result<Self> {
        let dir = tokio::task::spawn_blocking(|| {
            tempfile::Builder::new()
                .prefix(&current_prefix())
                .permissions(Permissions::from_mode(0o700))
                .tempdir()
        })
        .await
        .map_err(io::Error::other)??;
        let path = dir.path().to_path_buf();
        Ok(Self {
            dir: Some(dir),
            path,
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Creates one item's staging file, refusing to reuse an existing path.
    pub async fn open_item(&self, item_id: Uuid) -> io::Result<tokio::fs::File> {
        tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(batch_item_path(&self.path, item_id))
            .await
    }

    /// Removes the directory and every staged file it still holds.
    pub async fn remove(mut self) -> io::Result<()> {
        match self.dir.take() {
            Some(dir) => tokio::task::spawn_blocking(move || dir.close())
                .await
                .map_err(io::Error::other)?,
            None => Ok(()),
        }
    }
}

impl Drop for BatchStagingDir {
    fn drop(&mut self) {
        let Some(dir) = self.dir.take() else {
            return;
        };
        // Recursive removal blocks, so keep it off async workers when possible.
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                handle.spawn_blocking(move || {
                    if let Err(error) = dir.close()
                        && error.kind() != io::ErrorKind::NotFound
                    {
                        warn!(%error, "Could not remove a photograph batch staging directory");
                    }
                });
            }
            Err(_) => drop(dir),
        }
    }
}

pub fn batch_item_path(directory: &Path, item_id: Uuid) -> PathBuf {
    directory.join(format!("{item_id}.orig"))
}

pub async fn append_chunk(file: &mut tokio::fs::File, chunk: &[u8]) -> io::Result<()> {
    file.write_all(chunk).await
}

/// Removes staging directories that an earlier process left behind.
///
/// Only directories whose names start with the staging prefix and lack this
/// process's token are removed; symbolic links are never followed.
pub async fn sweep_stale_batch_dirs() -> io::Result<usize> {
    let root = std::env::temp_dir();
    let current = current_prefix();
    tokio::task::spawn_blocking(move || sweep_stale_in(&root, &current))
        .await
        .map_err(io::Error::other)?
}

fn sweep_stale_in(root: &Path, current_prefix: &str) -> io::Result<usize> {
    let mut removed = 0;
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            continue;
        };
        if !name.starts_with(BATCH_DIR_PREFIX) || name.starts_with(current_prefix) {
            continue;
        }
        if !entry.file_type()?.is_dir() {
            continue;
        }
        match std::fs::remove_dir_all(entry.path()) {
            Ok(()) => removed += 1,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                warn!(%error, path = %entry.path().display(), "Could not sweep a stale batch staging directory");
            }
        }
    }
    Ok(removed)
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::{BatchStagingDir, current_prefix, sweep_stale_in};

    #[tokio::test]
    async fn staging_directories_are_private_and_removed() -> std::io::Result<()> {
        let staging = BatchStagingDir::create().await?;
        let path = staging.path().to_path_buf();
        let mode = std::fs::metadata(&path)?.permissions().mode() & 0o777;
        assert_eq!(mode, 0o700);
        let item = uuid::Uuid::now_v7();
        drop(staging.open_item(item).await?);
        let file_mode = std::fs::metadata(super::batch_item_path(&path, item))?
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(file_mode, 0o600);
        staging.remove().await?;
        assert!(!path.exists());
        Ok(())
    }

    #[test]
    fn sweep_removes_only_other_processes_directories() -> std::io::Result<()> {
        let root = tempfile::tempdir()?;
        let current = current_prefix();
        let mine = root.path().join(format!("{current}abc"));
        let stale = root.path().join("cyhdev-batch-0123-old");
        let legacy = root.path().join("cyhdev-batch");
        let unrelated = root.path().join("other-dir");
        for dir in [&mine, &stale, &legacy, &unrelated] {
            std::fs::create_dir(dir)?;
        }
        assert_eq!(sweep_stale_in(root.path(), &current)?, 2);
        assert!(mine.exists() && unrelated.exists());
        assert!(!stale.exists() && !legacy.exists());
        Ok(())
    }
}
