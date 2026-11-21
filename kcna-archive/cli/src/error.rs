//! Typed failures for network, parsing, and durable storage operations.

use std::{io, path::PathBuf};

/// Project result type.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("filesystem operation failed for {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("invalid URL: {0}")]
    Url(#[from] url::ParseError),
    #[error("invalid selector: {0}")]
    Selector(String),
    #[error("required page field is missing: {0}")]
    MissingField(&'static str),
    #[error("crawl exceeded its configured limit of {limit} URLs")]
    CrawlLimit { limit: usize },
    #[error("archive is incomplete; {failures} requests failed after retries")]
    Incomplete { failures: usize },
    #[error("manifest serialization failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("hexadecimal digest is invalid: {0}")]
    Hex(#[from] hex::FromHexError),
    #[error("timestamp or date is invalid: {0}")]
    Time(#[from] time::error::Parse),
    #[error("archive record is invalid: {0}")]
    InvalidRecord(String),
    #[error("DATABASE_URL is not configured")]
    MissingDatabaseUrl,
    #[error("database connection failed: {0}")]
    DatabaseConnection(#[from] diesel::ConnectionError),
    #[error("database operation failed: {0}")]
    Database(#[from] diesel::result::Error),
    #[error("database pool failed: {0}")]
    DatabasePool(String),
    #[error("database migration failed: {0}")]
    Migration(String),
    #[error("blocking database task failed: {0}")]
    DatabaseTask(#[from] tokio::task::JoinError),
    #[error("response from {url} exceeds {limit} bytes")]
    ResponseTooLarge { url: String, limit: usize },
    #[error(transparent)]
    Snapshot(#[from] kcna_archive_core::error::Error),
    #[error("{url} was recently reported as nonexistent; not asking again yet")]
    KnownMissing { url: String },
    #[error("interrupted by a shutdown signal")]
    Interrupted,
}

impl Error {
    /// Returns true when repeating the same remote operation may succeed.
    ///
    /// Transport failures and overload statuses are transient on the archive
    /// hosts. A request that could not be built, a redirect loop, or a client
    /// error such as 404 fails identically on every attempt.
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Http(error) => match error.status() {
                Some(status) => retryable_status(status),
                None => !error.is_builder() && !error.is_redirect(),
            },
            Self::DatabaseConnection(_) => true,
            _ => false,
        }
    }
}

impl Error {
    /// Returns true when the remote host states that the page does not exist,
    /// now or in a recent run that recorded it.
    ///
    /// The mirror's own index pages link to thousands of pages it never
    /// served. Those are a property of the source, not a failed collection.
    pub fn is_missing(&self) -> bool {
        match self {
            Self::Http(error) => matches!(
                error.status().map(|status| status.as_u16()),
                Some(404 | 410)
            ),
            Self::KnownMissing { .. } => true,
            _ => false,
        }
    }
}

fn retryable_status(status: reqwest::StatusCode) -> bool {
    status.as_u16() == 429 || status.as_u16() == 408 || status.is_server_error()
}

/// Adds the affected path to an I/O failure.
pub fn io_error(path: impl Into<PathBuf>) -> impl FnOnce(io::Error) -> Error {
    let path = path.into();
    move |source| Error::Io { path, source }
}

#[cfg(test)]
mod tests {
    use reqwest::StatusCode;

    use super::*;

    #[test]
    fn retries_only_transient_statuses() {
        assert!(retryable_status(StatusCode::TOO_MANY_REQUESTS));
        assert!(retryable_status(StatusCode::BAD_GATEWAY));
        assert!(retryable_status(StatusCode::REQUEST_TIMEOUT));
        assert!(!retryable_status(StatusCode::NOT_FOUND));
        assert!(!retryable_status(StatusCode::FORBIDDEN));
    }

    #[test]
    fn local_failures_are_not_retried() {
        assert!(!Error::MissingField("_csrf").is_retryable());
        assert!(!Error::Interrupted.is_retryable());
        assert!(!Error::Interrupted.is_missing());
        let known = Error::KnownMissing {
            url: "http://www.kcna.co.jp/a.htm".to_owned(),
        };
        assert!(known.is_missing());
        assert!(!known.is_retryable());
    }
}
