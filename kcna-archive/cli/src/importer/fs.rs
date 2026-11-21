//! Deterministic bounded filesystem listing helpers.

use std::path::{Path, PathBuf};

use crate::error::{Result, io_error};

/// Lists the regular files directly inside `root`, sorted by path.
pub async fn files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut entries = tokio::fs::read_dir(root).await.map_err(io_error(root))?;
    let mut paths = Vec::new();
    while let Some(entry) = entries.next_entry().await.map_err(io_error(root))? {
        let file_type = entry.file_type().await.map_err(io_error(entry.path()))?;
        if file_type.is_file() {
            paths.push(entry.path());
        }
    }
    paths.sort_unstable();
    Ok(paths)
}
