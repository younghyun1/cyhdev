//! Failures of the snapshot and the search index.

use std::{io, path::PathBuf};

/// Result type of this crate.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("filesystem operation failed for {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("search index failed: {0}")]
    Search(String),
    #[error("archive snapshot is invalid: {0}")]
    Snapshot(String),
    #[error("snapshot manifest could not be read or written: {0}")]
    Manifest(#[from] serde_json::Error),
}

/// Attaches the path an I/O failure happened on.
pub fn io_error(path: impl Into<PathBuf>) -> impl FnOnce(io::Error) -> Error {
    let path = path.into();
    move |source| Error::Io { path, source }
}

/// Converts an index failure, keeping its message.
pub fn search_error(error: tantivy::TantivyError) -> Error {
    Error::Search(error.to_string())
}
