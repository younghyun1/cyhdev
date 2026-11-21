//! Failures of the archive feature, free of HTTP concerns.

use std::fmt;

/// Why the snapshot cannot answer right now.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnavailableReason {
    /// The snapshot directory or its manifest does not exist on this machine.
    NotInstalled,
    /// The snapshot is still being read into memory.
    Loading,
    /// The snapshot exists but could not be loaded; the cause is in the logs.
    Failed,
}

impl fmt::Display for UnavailableReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::NotInstalled => "not installed",
            Self::Loading => "loading",
            Self::Failed => "failed to load",
        })
    }
}

/// One failure source per variant; the API layer maps them to HTTP.
#[derive(Debug, thiserror::Error)]
pub enum ArchiveError {
    /// A parameter failed validation.
    #[error("invalid archive request")]
    BadRequest,
    /// The requested document is not in the snapshot.
    #[error("archive document not found")]
    NotFound,
    /// No snapshot is available to answer from.
    #[error("archive snapshot is unavailable: {0}")]
    Unavailable(UnavailableReason),
    /// Another snapshot load or reload is running.
    #[error("an archive snapshot reload is already in progress")]
    Busy,
    /// The reload failed; the previous snapshot, if any, keeps serving.
    #[error("archive snapshot reload failed; the previous snapshot keeps serving")]
    ReloadFailed,
    /// An index or task failure; the cause is logged where it happened.
    #[error("archive query failed internally")]
    Internal,
}

impl ArchiveError {
    /// True when the same request can succeed later without any change.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Unavailable(UnavailableReason::Loading) | Self::Busy
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_transient_failures_are_retryable() {
        assert!(ArchiveError::Busy.is_retryable());
        assert!(ArchiveError::Unavailable(UnavailableReason::Loading).is_retryable());
        for error in [
            ArchiveError::BadRequest,
            ArchiveError::NotFound,
            ArchiveError::Unavailable(UnavailableReason::NotInstalled),
            ArchiveError::Unavailable(UnavailableReason::Failed),
            ArchiveError::ReloadFailed,
            ArchiveError::Internal,
        ] {
            assert!(!error.is_retryable(), "{error}");
        }
    }
}
