//! The case-insensitive identity migration refuses existing collisions without changing data.

mod support;

use diesel::{Connection, RunQueryDsl, pg::PgConnection};
use diesel_migrations::MigrationHarness;

use rust_be_template::init::db_migrations::MIGRATIONS;

use support::{
    database::{DatabaseTestFuture, TestDatabase, TestResult, require, run_database_test},
    migrations::rewind_to_migration,
};

const CASE_INSENSITIVE_IDENTITY: &str = "202609231000000000";
const SYSTEM_ACTOR: &str = "00000000-0000-0000-0000-000000000000";

#[tokio::test]
#[ignore = "requires explicit safe TEST_DATABASE_URL and PostgreSQL 18"]
async fn identity_migration_refuses_case_collisions() -> TestResult {
    run_database_test(collision_case).await
}

fn collision_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let database_url = database.database_url().to_owned();
        tokio::task::spawn_blocking(move || -> TestResult {
            let mut connection = PgConnection::establish(&database_url)?;
            let previous = previous_migration(&mut connection)?;
            rewind_to_migration(&mut connection, &previous)?;

            insert_user(&mut connection, "CollideA", "Collide@example.test")?;
            insert_user(&mut connection, "CollideB", "collide@EXAMPLE.test")?;
            expect_refusal(&mut connection, "users_user_email_lower_unique")?;
            diesel::sql_query("DELETE FROM users WHERE user_name = 'CollideB'")
                .execute(&mut connection)?;

            insert_user(&mut connection, "NameClash", "name-a@example.test")?;
            insert_user(&mut connection, "nameclash", "name-b@example.test")?;
            expect_refusal(&mut connection, "users_user_name_lower_unique")?;
            diesel::sql_query("DELETE FROM users WHERE user_name = 'nameclash'")
                .execute(&mut connection)?;

            connection.run_pending_migrations(MIGRATIONS)?;
            require(
                connection.pending_migrations(MIGRATIONS)?.is_empty(),
                "identity migration did not apply after collisions were resolved",
            )
        })
        .await?
    })
}

/// The newest embedded migration older than the identity migration.
fn previous_migration(connection: &mut PgConnection) -> TestResult<String> {
    let previous = connection
        .applied_migrations()?
        .into_iter()
        .map(|version| version.to_string())
        .filter(|version| version.as_str() < CASE_INSENSITIVE_IDENTITY)
        .max();
    match previous {
        Some(previous) => Ok(previous),
        None => {
            require(false, "no migration precedes the identity migration")?;
            Ok(String::new())
        }
    }
}

fn insert_user(connection: &mut PgConnection, user_name: &str, user_email: &str) -> TestResult {
    diesel::sql_query(format!(
        "INSERT INTO users (user_name, user_email, user_password_hash, user_country, user_language) \
         SELECT '{user_name}', '{user_email}', user_password_hash, user_country, user_language \
         FROM users WHERE user_id = '{SYSTEM_ACTOR}'"
    ))
    .execute(connection)?;
    Ok(())
}

fn expect_refusal(connection: &mut PgConnection, index_name: &str) -> TestResult {
    match connection.run_pending_migrations(MIGRATIONS) {
        Ok(_) => require(false, "identity migration accepted colliding rows"),
        Err(error) => require(
            error.to_string().contains(index_name),
            "identity migration refused collisions without naming the index",
        ),
    }
}
