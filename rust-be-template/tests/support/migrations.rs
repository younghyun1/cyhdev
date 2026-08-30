//! Helpers for targeting one embedded migration without assuming it is latest.

use diesel::pg::PgConnection;
use diesel_migrations::MigrationHarness;

use rust_be_template::init::db_migrations::MIGRATIONS;

use super::database::{TestResult, require};

pub fn rewind_to_migration(connection: &mut PgConnection, target: &str) -> TestResult {
    loop {
        let applied = connection.applied_migrations()?;
        let target_is_applied = applied.iter().any(|version| version.to_string() == target);
        require(target_is_applied, "target migration is not applied")?;
        let latest = applied
            .into_iter()
            .max_by_key(|version| version.to_string());
        let latest = match latest {
            Some(latest) => latest,
            None => return require(false, "migration chain is unexpectedly empty"),
        };
        if latest.to_string() == target {
            return Ok(());
        }
        connection.revert_last_migration(MIGRATIONS)?;
    }
}

pub fn latest_down_is_refused(connection: &mut PgConnection) -> TestResult {
    require(
        connection.revert_last_migration(MIGRATIONS).is_err(),
        "protected migration rollback unexpectedly succeeded",
    )
}
