//! Account migrations refuse identity collisions and convert stored token capabilities.

mod support;

use diesel::{Connection, QueryDsl, RunQueryDsl, pg::PgConnection};
use diesel_migrations::MigrationHarness;
use sha2::{Digest, Sha256};

use rust_be_template::{init::db_migrations::MIGRATIONS, schema::email_verification_tokens};

use support::{
    database::{DatabaseTestFuture, TestDatabase, TestResult, require, run_database_test},
    migrations::rewind_to_migration,
};

const CASE_INSENSITIVE_IDENTITY: &str = "202609231000000000";
const CAPABILITY_TOKEN_DIGESTS: &str = "202609231010000000";
const LEGACY_TOKEN: &str = "0190f1c0-7c4e-7b1a-8f00-0123456789ab";
const SYSTEM_ACTOR: &str = "00000000-0000-0000-0000-000000000000";

#[tokio::test]
#[ignore = "requires explicit safe TEST_DATABASE_URL and PostgreSQL 18"]
async fn identity_migration_refuses_case_collisions() -> TestResult {
    run_database_test(collision_case).await
}

#[tokio::test]
#[ignore = "requires explicit safe TEST_DATABASE_URL and PostgreSQL 18"]
async fn token_digest_migration_hashes_existing_token_text() -> TestResult {
    run_database_test(token_digest_case).await
}

fn token_digest_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let database_url = database.database_url().to_owned();
        tokio::task::spawn_blocking(move || -> TestResult {
            let mut connection = PgConnection::establish(&database_url)?;
            let previous = previous_migration(&mut connection, CAPABILITY_TOKEN_DIGESTS)?;
            rewind_to_migration(&mut connection, &previous)?;
            insert_user(&mut connection, "LegacyToken", "legacy-token@example.test")?;
            diesel::sql_query(format!(
                "INSERT INTO email_verification_tokens (user_id, email_verification_token, email_verification_token_expires_at) \
                 SELECT user_id, '{LEGACY_TOKEN}', now() + INTERVAL '1 day' FROM users WHERE user_name = 'LegacyToken'"
            ))
            .execute(&mut connection)?;
            connection.run_pending_migrations(MIGRATIONS)?;

            let stored = email_verification_tokens::table
                .select(email_verification_tokens::email_verification_token_hash)
                .load::<Vec<u8>>(&mut connection)?;
            let expected: [u8; 32] = Sha256::digest(LEGACY_TOKEN.as_bytes()).into();
            require(
                stored == vec![expected.to_vec()],
                "token migration did not store the SHA-256 of the existing token text",
            )?;
            // Rolling back with live rows must succeed and simply invalidate the links.
            rewind_to_migration(&mut connection, &previous)?;
            connection.run_pending_migrations(MIGRATIONS)?;
            require(
                connection.pending_migrations(MIGRATIONS)?.is_empty(),
                "token migration did not round-trip",
            )
        })
        .await?
    })
}

fn collision_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let database_url = database.database_url().to_owned();
        tokio::task::spawn_blocking(move || -> TestResult {
            let mut connection = PgConnection::establish(&database_url)?;
            let previous = previous_migration(&mut connection, CASE_INSENSITIVE_IDENTITY)?;
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

/// The newest applied migration older than `target`.
fn previous_migration(connection: &mut PgConnection, target: &str) -> TestResult<String> {
    let previous = connection
        .applied_migrations()?
        .into_iter()
        .map(|version| version.to_string())
        .filter(|version| version.as_str() < target)
        .max();
    match previous {
        Some(previous) => Ok(previous),
        None => {
            require(false, "no migration precedes the target migration")?;
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
