//! Error types shared by PostgreSQL integration-test support.

use std::{env, error::Error};

use diesel_async::pooled_connection::PoolError;
use thiserror::Error;

pub type BoxError = Box<dyn Error + Send + Sync + 'static>;
pub type TestResult<T = ()> = Result<T, BoxError>;

/// Failures produced while setting up, running, or cleaning an integration test.
#[derive(Debug, Error)]
pub enum HarnessError {
    #[error("TEST_DATABASE_URL must name an existing PostgreSQL maintenance database")]
    MissingDatabaseUrl(#[source] env::VarError),
    #[error("TEST_DATABASE_URL is not a valid URL: {reason}")]
    InvalidDatabaseUrl { reason: String },
    #[error("TEST_DATABASE_URL must use postgresql or postgres, not {scheme}")]
    UnsupportedDatabaseScheme { scheme: String },
    #[error("TEST_DATABASE_URL must include a maintenance database name")]
    MissingMaintenanceDatabase,
    #[error("generated integration database name is invalid: {database_name}")]
    InvalidGeneratedDatabaseName { database_name: String },
    #[error("could not connect to PostgreSQL while attempting to {action}")]
    LifecycleConnection {
        action: &'static str,
        #[source]
        source: diesel::ConnectionError,
    },
    #[error("PostgreSQL failed to {action}")]
    LifecycleStatement {
        action: &'static str,
        #[source]
        source: diesel::result::Error,
    },
    #[error("embedded migrations failed in the isolated integration database")]
    Migrations(#[source] anyhow::Error),
    #[error("could not open the isolated integration database pool")]
    Pool(#[source] PoolError),
    #[error("blocking PostgreSQL lifecycle task failed")]
    LifecycleTask(#[source] tokio::task::JoinError),
    #[error("integration database was already cleaned")]
    DatabaseAlreadyCleaned,
    #[error("test body panicked: {message}")]
    TestPanicked { message: String },
    #[error("setup failed and cleanup also failed: {cleanup}")]
    SetupAndCleanup {
        #[source]
        setup: Box<HarnessError>,
        cleanup: Box<HarnessError>,
    },
    #[error("test failed and cleanup also failed: {cleanup}")]
    TestAndCleanup {
        #[source]
        test: BoxError,
        cleanup: Box<HarnessError>,
    },
    #[error("integration assertion failed: {message}")]
    Assertion { message: &'static str },
}
