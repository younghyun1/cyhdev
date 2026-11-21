//! Where the parts of a snapshot live below its directory.
//!
//! A snapshot is a directory holding `snapshot.json` and the index in
//! `index/`. It is self-contained and relocatable: copying the directory to
//! another machine is a complete deployment of the archive.

use std::path::{Path, PathBuf};

/// Manifest file name inside a snapshot directory.
pub const MANIFEST_FILE: &str = "snapshot.json";
const INDEX_DIRECTORY: &str = "index";

/// Location of the search index inside a snapshot directory.
pub fn index_directory(snapshot: &Path) -> PathBuf {
    snapshot.join(INDEX_DIRECTORY)
}

/// Location of the manifest inside a snapshot directory.
pub fn manifest_path(snapshot: &Path) -> PathBuf {
    snapshot.join(MANIFEST_FILE)
}
