//! Targeted evidence for fail-closed migration rollback guards.

mod support;

use diesel::{Connection, RunQueryDsl, pg::PgConnection};

use support::{
    database::{DatabaseTestFuture, TestDatabase, TestResult, run_database_test},
    migrations::{latest_down_is_refused, rewind_to_migration},
};

const LIFECYCLE: &str = "202608300100000000";
const OIDC: &str = "202608300200000000";
const RETENTION: &str = "202608300400000000";
const AUTHORIZATION: &str = "202608300500000000";
const FORUM: &str = "202608300600000000";

#[tokio::test]
#[ignore = "requires explicit safe TEST_DATABASE_URL and PostgreSQL 18"]
async fn lifecycle_down_refuses_durable_cleanup_state() -> TestResult {
    run_database_test(lifecycle_case).await
}

#[tokio::test]
#[ignore = "requires explicit safe TEST_DATABASE_URL and PostgreSQL 18"]
async fn oidc_down_refuses_linked_identity_state() -> TestResult {
    run_database_test(oidc_case).await
}

#[tokio::test]
#[ignore = "requires explicit safe TEST_DATABASE_URL and PostgreSQL 18"]
async fn retention_down_refuses_delivery_ledger_state() -> TestResult {
    run_database_test(retention_case).await
}

#[tokio::test]
#[ignore = "requires explicit safe TEST_DATABASE_URL and PostgreSQL 18"]
async fn authorization_down_refuses_audit_state() -> TestResult {
    run_database_test(authorization_case).await
}

#[tokio::test]
#[ignore = "requires explicit safe TEST_DATABASE_URL and PostgreSQL 18"]
async fn forum_down_refuses_retained_content() -> TestResult {
    run_database_test(forum_case).await
}

fn lifecycle_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    guard_case(database, LIFECYCLE, seed_lifecycle)
}
fn oidc_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    guard_case(database, OIDC, seed_oidc)
}
fn retention_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    guard_case(database, RETENTION, seed_retention)
}
fn authorization_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    guard_case(database, AUTHORIZATION, seed_authorization)
}
fn forum_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    guard_case(database, FORUM, seed_forum)
}

fn guard_case<'a>(
    database: &'a TestDatabase,
    target: &'static str,
    seed: fn(&mut PgConnection) -> TestResult,
) -> DatabaseTestFuture<'a> {
    Box::pin(async move {
        let database_url = database.database_url().to_owned();
        tokio::task::spawn_blocking(move || -> TestResult {
            let mut connection = PgConnection::establish(&database_url)?;
            rewind_to_migration(&mut connection, target)?;
            seed(&mut connection)?;
            latest_down_is_refused(&mut connection)
        })
        .await?
    })
}

fn seed_lifecycle(connection: &mut PgConnection) -> TestResult {
    diesel::sql_query(
        "INSERT INTO media_object_cleanup (media_object_cleanup_bucket, media_object_cleanup_key, media_object_cleanup_original_url, media_object_cleanup_reason, media_object_cleanup_source_id) VALUES ('guard-bucket', 'guard-object', 's3://guard-bucket/guard-object', 'deleted_photograph_image', '019d0000-0000-7000-8000-00000000a001')",
    )
    .execute(connection)?;
    Ok(())
}

fn seed_oidc(connection: &mut PgConnection) -> TestResult {
    diesel::sql_query(
        "INSERT INTO account_oidc_identities (account_oidc_identity_user_id, account_oidc_identity_issuer, account_oidc_identity_subject, account_oidc_identity_provider_email) VALUES ('00000000-0000-0000-0000-000000000000', 'https://guard.example', 'guard-subject', 'guard@example.test')",
    )
    .execute(connection)?;
    Ok(())
}

fn seed_retention(connection: &mut PgConnection) -> TestResult {
    diesel::sql_query(
        "INSERT INTO users (user_id, user_name, user_email, user_password_hash, user_created_at, user_updated_at, user_is_email_verified, user_country, user_language, user_subdivision, user_deleted_at, user_purge_after, user_hard_purged_at, user_is_system_actor) SELECT '019d0000-0000-7000-8000-00000000a002', 'RetentionGuard', 'retention-guard@example.test', user_password_hash, now(), now(), TRUE, user_country, user_language, NULL, now(), now() + INTERVAL '30 days', NULL, FALSE FROM users WHERE user_id = '00000000-0000-0000-0000-000000000000'",
    )
    .execute(&mut *connection)?;
    diesel::sql_query(
        "INSERT INTO deleted_account_retention (deleted_account_retention_user_id, deleted_account_retention_user_name, deleted_account_retention_email, deleted_account_retention_country, deleted_account_retention_language) SELECT user_id, user_name, user_email, user_country, user_language FROM users WHERE user_id = '019d0000-0000-7000-8000-00000000a002'",
    )
    .execute(&mut *connection)?;
    diesel::sql_query(
        "INSERT INTO account_retention_notifications (account_retention_notification_user_id, account_retention_notification_stage, account_retention_notification_scheduled_for, account_retention_notification_next_attempt_at) SELECT user_id, 'seven_days_before_purge'::account_retention_notification_stage, user_purge_after - INTERVAL '7 days', user_purge_after - INTERVAL '7 days' FROM users WHERE user_id = '019d0000-0000-7000-8000-00000000a002'",
    )
    .execute(connection)?;
    Ok(())
}

fn seed_authorization(connection: &mut PgConnection) -> TestResult {
    diesel::sql_query(
        "INSERT INTO authorization_audit_events (authorization_audit_event_actor_user_id, authorization_audit_event_kind, authorization_audit_event_target_user_id, authorization_audit_event_role_id, authorization_audit_event_role_name, authorization_audit_event_old_value, authorization_audit_event_new_value, authorization_audit_event_reason) VALUES ('00000000-0000-0000-0000-000000000000', 'user_role_assigned', '00000000-0000-0000-0000-000000000000', '019a6c86-8bca-7b91-b9c0-1d4cc96b3263', 'Younghyun', 'user', 'younghyun', 'Protect authorization rollback guard')",
    )
    .execute(connection)?;
    Ok(())
}

fn seed_forum(connection: &mut PgConnection) -> TestResult {
    diesel::sql_query(
        "INSERT INTO forum_topics (forum_topic_author_user_id, forum_topic_title, forum_topic_body) VALUES ('00000000-0000-0000-0000-000000000000', 'Rollback guard topic', 'Retained forum content')",
    )
    .execute(connection)?;
    Ok(())
}
