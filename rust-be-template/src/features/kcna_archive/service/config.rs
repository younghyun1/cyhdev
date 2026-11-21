//! Where the snapshot lives, read from the environment.

use std::{ffi::OsString, path::PathBuf};

/// Environment variable naming the snapshot directory.
pub const ARCHIVE_DIR_ENV: &str = "KCNA_ARCHIVE_DIR";
/// Snapshot directory when the variable is unset, relative to the backend
/// working directory like the other runtime data paths.
pub const DEFAULT_ARCHIVE_DIR: &str = "data/kcna_archive";

/// Runtime configuration of the archive service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KcnaArchiveConfig {
    /// Directory holding `snapshot.json` and the `index/` directory.
    pub directory: PathBuf,
}

impl KcnaArchiveConfig {
    /// Reads `KCNA_ARCHIVE_DIR`; an unset or empty value selects the default.
    pub fn from_environment() -> Self {
        Self::from_value(std::env::var_os(ARCHIVE_DIR_ENV))
    }

    fn from_value(value: Option<OsString>) -> Self {
        let directory = match value {
            Some(value) if !value.is_empty() => PathBuf::from(value),
            Some(_) | None => PathBuf::from(DEFAULT_ARCHIVE_DIR),
        };
        Self { directory }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unset_or_empty_selects_the_default_directory() {
        let default = PathBuf::from(DEFAULT_ARCHIVE_DIR);
        assert_eq!(KcnaArchiveConfig::from_value(None).directory, default);
        assert_eq!(
            KcnaArchiveConfig::from_value(Some(OsString::new())).directory,
            default
        );
    }

    #[test]
    fn a_configured_directory_is_used_as_given() {
        let config = KcnaArchiveConfig::from_value(Some(OsString::from("/srv/kcna")));
        assert_eq!(config.directory, PathBuf::from("/srv/kcna"));
    }
}
