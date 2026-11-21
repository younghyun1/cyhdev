//! Small bounded PostgreSQL pool for batch ingestion.

use std::time::Duration;

use diesel_async::{
    AsyncPgConnection,
    pooled_connection::{AsyncDieselConnectionManager, bb8::Pool},
};

use crate::error::{Error, Result};

pub type PgPool = Pool<AsyncPgConnection>;

/// Opens a small importer pool sized for concurrent legacy and live batch writers.
pub async fn connect(database_url: &str) -> Result<PgPool> {
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    Pool::builder()
        .min_idle(Some(2))
        .max_size(4)
        .connection_timeout(Duration::from_secs(2))
        .build(manager)
        .await
        .map_err(|error| Error::DatabasePool(error.to_string()))
}

/// Reads the connection string every database command requires.
pub fn database_url() -> Result<String> {
    std::env::var("DATABASE_URL").map_err(|_| Error::MissingDatabaseUrl)
}
