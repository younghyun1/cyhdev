//! Corpus facts recorded beside the index when a snapshot is written.
//!
//! The index can count what it holds, but not what the collector's database
//! knows beyond it: how many stored documents were folded into another
//! source's copy, how many translation groups exist, and when the corpus was
//! last retrieved. The manifest carries those so the status page needs no
//! database.

use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::{
    error::{Error, Result, io_error},
    snapshot::layout::manifest_path,
};

/// Version of the snapshot layout. A server refuses a snapshot written for
/// another version instead of misreading it.
pub const FORMAT: u32 = 1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Manifest {
    pub format: u32,
    /// RFC 3339 time the snapshot was last written.
    pub built_at: String,
    /// Documents stored in the collector's database.
    pub documents: u64,
    /// Of those, duplicates folded into another source's copy.
    pub copies: u64,
    pub translation_groups: u64,
    /// RFC 3339 time of the newest stored retrieval.
    pub last_retrieved_at: Option<String>,
}

impl Manifest {
    /// Reads and validates the manifest of the snapshot in `snapshot`.
    pub fn read(snapshot: &Path) -> Result<Self> {
        let path = manifest_path(snapshot);
        let bytes = fs::read(&path).map_err(io_error(&path))?;
        let manifest: Self = serde_json::from_slice(&bytes)?;
        if manifest.format != FORMAT {
            return Err(Error::Snapshot(format!(
                "snapshot format {} is not the supported format {FORMAT}",
                manifest.format
            )));
        }
        Ok(manifest)
    }

    /// Writes the manifest, replacing an earlier one atomically so a reader
    /// never sees a partial file.
    pub fn write(&self, snapshot: &Path) -> Result<()> {
        fs::create_dir_all(snapshot).map_err(io_error(snapshot))?;
        let path = manifest_path(snapshot);
        let staged = path.with_extension("json.tmp");
        let bytes = serde_json::to_vec_pretty(self)?;
        fs::write(&staged, bytes).map_err(io_error(&staged))?;
        fs::rename(&staged, &path).map_err(io_error(&path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(format: u32) -> Manifest {
        Manifest {
            format,
            built_at: "2026-11-21T12:00:00Z".to_owned(),
            documents: 12,
            copies: 2,
            translation_groups: 3,
            last_retrieved_at: None,
        }
    }

    #[test]
    fn round_trips_and_rejects_other_formats() -> Result<()> {
        let directory = tempfile::tempdir().map_err(io_error("tempdir"))?;
        let written = manifest(FORMAT);
        written.write(directory.path())?;
        assert_eq!(Manifest::read(directory.path())?, written);
        manifest(FORMAT + 1).write(directory.path())?;
        assert!(matches!(
            Manifest::read(directory.path()),
            Err(Error::Snapshot(_))
        ));
        Ok(())
    }

    #[test]
    fn a_missing_manifest_is_an_io_failure() {
        let missing = Path::new("/nonexistent/kcna-archive-snapshot");
        assert!(matches!(Manifest::read(missing), Err(Error::Io { .. })));
    }
}
