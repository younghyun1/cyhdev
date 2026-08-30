//! Error types for configuration, execution, and regression failures.

use std::{io, path::PathBuf};

use thiserror::Error;

pub type HarnessResult<T> = Result<T, HarnessError>;

#[derive(Debug, Error)]
pub enum HarnessError {
    #[error("invalid arguments: {0}")]
    Arguments(String),
    #[error("invalid configuration in {path}: {detail}")]
    Configuration { path: PathBuf, detail: String },
    #[error("failed to {operation} {path}: {source}")]
    Io {
        operation: &'static str,
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("invalid JSON in {path}: {source}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("input {path} exceeds the {maximum_bytes}-byte limit")]
    InputTooLarge {
        path: PathBuf,
        maximum_bytes: u64,
    },
    #[error("could not initialize structured logging: {0}")]
    Logging(String),
    #[error("could not resolve HTTP target `{target}`: {source}")]
    Resolve {
        target: String,
        #[source]
        source: io::Error,
    },
    #[error("could not spawn measurement worker {worker}: {source}")]
    ThreadSpawn {
        worker: usize,
        #[source]
        source: io::Error,
    },
    #[error("measurement worker {worker} terminated unexpectedly")]
    WorkerTerminated { worker: usize },
    #[error("measurement worker {worker} found an invalid workload invariant: {detail}")]
    WorkerInvariant { worker: usize, detail: String },
    #[error("throughput regression: {0}")]
    Regression(String),
}

impl HarnessError {
    /// Indicates failures worth retrying without changing inputs.
    pub const fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Io { .. }
                | Self::Resolve { .. }
                | Self::ThreadSpawn { .. }
                | Self::WorkerTerminated { .. }
        )
    }
}
