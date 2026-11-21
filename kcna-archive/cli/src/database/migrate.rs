//! Embedded migration runner on an isolated PostgreSQL connection.

use diesel_async::{
    AsyncConnection, AsyncPgConnection, async_connection_wrapper::AsyncConnectionWrapper,
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use tokio::task;
use tracing::info;

use crate::error::{Error, Result};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Applies pending migrations without occupying an importer pool connection.
pub async fn run(database_url: &str) -> Result<()> {
    let connection = AsyncPgConnection::establish(database_url).await?;
    let applied = task::spawn_blocking(move || {
        let mut wrapper: AsyncConnectionWrapper<AsyncPgConnection> = connection.into();
        wrapper
            .run_pending_migrations(MIGRATIONS)
            .map(|versions| versions.len())
            .map_err(|error| Error::Migration(error.to_string()))
    })
    .await??;
    info!(applied, "Database migrations applied");
    Ok(())
}
