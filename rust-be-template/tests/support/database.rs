//! Parallel-safe PostgreSQL lifecycle support for integration tests.

use std::{
    any::Any,
    env,
    future::Future,
    panic::AssertUnwindSafe,
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use diesel::{Connection, RunQueryDsl, pg::PgConnection};
use diesel_async::{
    AsyncPgConnection,
    pooled_connection::{AsyncDieselConnectionManager, bb8::Pool},
};
use futures_util::FutureExt;
use reqwest::Url;
use tracing::warn;
use uuid::Uuid;

use rust_be_template::init::db_migrations::run_pending_migrations;

pub use super::error::{BoxError, HarnessError, TestResult};

const DATABASE_ENV: &str = "TEST_DATABASE_URL";
const DATABASE_PREFIX: &str = "cyhdev_it_";
static DATABASE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub type DatabaseTestFuture<'a> = Pin<Box<dyn Future<Output = TestResult> + Send + 'a>>;

/// An isolated database created on the explicitly configured test server.
pub struct TestDatabase {
    maintenance_url: String,
    database_url: String,
    database_name: String,
    pool: Option<Pool<AsyncPgConnection>>,
    cleaned: bool,
}

impl TestDatabase {
    async fn create() -> Result<Self, HarnessError> {
        let maintenance_url = configured_maintenance_url()?;
        let database_name = generated_database_name()?;
        run_lifecycle_statement(
            maintenance_url.as_str().to_owned(),
            "create isolated integration database",
            format!("CREATE DATABASE \"{database_name}\" TEMPLATE template0"),
        )
        .await?;

        let mut isolated_url = maintenance_url.clone();
        isolated_url.set_path(&format!("/{database_name}"));
        isolated_url.set_fragment(None);
        let database_url = isolated_url.as_str().to_owned();

        if let Err(source) = run_pending_migrations(database_url.clone()).await {
            let setup = HarnessError::Migrations(source);
            return cleanup_failed_setup(maintenance_url, database_name, setup).await;
        }

        let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url.clone());
        let pool = match Pool::builder()
            .min_idle(Some(0))
            .max_size(4)
            .test_on_check_out(true)
            .connection_timeout(Duration::from_secs(5))
            .build(manager)
            .await
        {
            Ok(pool) => pool,
            Err(source) => {
                let setup = HarnessError::Pool(source);
                return cleanup_failed_setup(maintenance_url, database_name, setup).await;
            }
        };

        Ok(Self {
            maintenance_url: maintenance_url.as_str().to_owned(),
            database_url,
            database_name,
            pool: Some(pool),
            cleaned: false,
        })
    }

    /// Clone the pool used by public repositories and services under test.
    pub fn pool(&self) -> Result<Pool<AsyncPgConnection>, HarnessError> {
        match &self.pool {
            Some(pool) => Ok(pool.clone()),
            None => Err(HarnessError::DatabaseAlreadyCleaned),
        }
    }

    /// URL of the generated database, used only by the migration harness.
    pub fn database_url(&self) -> &str {
        &self.database_url
    }

    async fn cleanup(mut self) -> Result<(), HarnessError> {
        let pool = match self.pool.take() {
            Some(pool) => pool,
            None => return Err(HarnessError::DatabaseAlreadyCleaned),
        };
        drop(pool);

        run_lifecycle_statement(
            self.maintenance_url.clone(),
            "drop isolated integration database",
            format!(
                "DROP DATABASE IF EXISTS \"{}\" WITH (FORCE)",
                self.database_name
            ),
        )
        .await?;
        self.cleaned = true;
        Ok(())
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        if !self.cleaned {
            warn!(
                database_name = %self.database_name,
                "Integration database cleanup did not complete"
            );
        }
    }
}

/// Run a test case and always attempt database cleanup, including after a panic.
pub async fn run_database_test<F>(test: F) -> TestResult
where
    F: for<'a> FnOnce(&'a TestDatabase) -> DatabaseTestFuture<'a>,
{
    let database = TestDatabase::create().await?;
    let outcome = AssertUnwindSafe(test(&database)).catch_unwind().await;
    let test_result = match outcome {
        Ok(result) => result,
        Err(payload) => Err(Box::new(HarnessError::TestPanicked {
            message: panic_message(payload),
        }) as BoxError),
    };
    let cleanup_result = database.cleanup().await;

    match (test_result, cleanup_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(test), Ok(())) => Err(test),
        (Ok(()), Err(cleanup)) => Err(Box::new(cleanup)),
        (Err(test), Err(cleanup)) => Err(Box::new(HarnessError::TestAndCleanup {
            test,
            cleanup: Box::new(cleanup),
        })),
    }
}

/// Return a test error without introducing a panic path.
pub fn require(condition: bool, message: &'static str) -> TestResult {
    if condition {
        Ok(())
    } else {
        Err(Box::new(HarnessError::Assertion { message }))
    }
}

fn configured_maintenance_url() -> Result<Url, HarnessError> {
    let raw_url = env::var(DATABASE_ENV).map_err(HarnessError::MissingDatabaseUrl)?;
    let parsed = Url::parse(&raw_url).map_err(|source| HarnessError::InvalidDatabaseUrl {
        reason: source.to_string(),
    })?;
    drop(raw_url);

    match parsed.scheme() {
        "postgresql" | "postgres" => {}
        scheme => {
            return Err(HarnessError::UnsupportedDatabaseScheme {
                scheme: scheme.to_owned(),
            });
        }
    }

    if parsed.path().trim_matches('/').is_empty() {
        return Err(HarnessError::MissingMaintenanceDatabase);
    }

    Ok(parsed)
}

fn generated_database_name() -> Result<String, HarnessError> {
    let sequence = DATABASE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let random_suffix = Uuid::new_v4().as_u128() as u64;
    let database_name = format!(
        "{DATABASE_PREFIX}{}_{}_{random_suffix:016x}",
        std::process::id(),
        sequence
    );
    let is_valid = database_name.len() <= 63
        && database_name.starts_with(DATABASE_PREFIX)
        && database_name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_');

    if is_valid {
        Ok(database_name)
    } else {
        Err(HarnessError::InvalidGeneratedDatabaseName { database_name })
    }
}

async fn cleanup_failed_setup<T>(
    maintenance_url: Url,
    database_name: String,
    setup: HarnessError,
) -> Result<T, HarnessError> {
    match drop_database(maintenance_url.as_str().to_owned(), database_name).await {
        Ok(()) => Err(setup),
        Err(cleanup) => Err(HarnessError::SetupAndCleanup {
            setup: Box::new(setup),
            cleanup: Box::new(cleanup),
        }),
    }
}

async fn drop_database(maintenance_url: String, database_name: String) -> Result<(), HarnessError> {
    run_lifecycle_statement(
        maintenance_url,
        "drop isolated integration database after failed setup",
        format!("DROP DATABASE IF EXISTS \"{database_name}\" WITH (FORCE)"),
    )
    .await
}

async fn run_lifecycle_statement(
    maintenance_url: String,
    action: &'static str,
    statement: String,
) -> Result<(), HarnessError> {
    // PostgreSQL exposes database lifecycle commands outside the query-builder
    // model; generated names are validated before this test-only DDL is built.
    tokio::task::spawn_blocking(move || {
        let mut connection = PgConnection::establish(&maintenance_url)
            .map_err(|source| HarnessError::LifecycleConnection { action, source })?;
        diesel::sql_query(statement)
            .execute(&mut connection)
            .map_err(|source| HarnessError::LifecycleStatement { action, source })?;
        Ok(())
    })
    .await
    .map_err(HarnessError::LifecycleTask)?
}

fn panic_message(payload: Box<dyn Any + Send>) -> String {
    match payload.downcast_ref::<&str>() {
        Some(message) => (*message).to_owned(),
        None => match payload.downcast_ref::<String>() {
            Some(message) => message.clone(),
            None => "non-string panic payload".to_owned(),
        },
    }
}
