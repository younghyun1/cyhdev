//! The archive reader list: empty by default, durable, limited to active accounts.

use std::sync::Arc;

use crate::support::{
    database::{DatabaseTestFuture, TestDatabase, TestResult, require, run_database_test},
    fixtures::{account_test_context, seed_account},
};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use rust_be_template::{
    features::kcna_archive::{
        error::ArchiveError,
        repository::access::ReaderRepository,
        service::{access::ReaderStore, archive_service::KcnaArchiveService},
    },
    schema::users,
};
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn the_reader_list_is_durable_and_holds_only_active_accounts() -> TestResult {
    run_database_test(reader_case).await
}

fn reader_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let first = seed_account(&context, "ArchiveReaderOne").await?;
        let second = seed_account(&context, "ArchiveReaderTwo").await?;
        let repository = ReaderRepository::new(context.pool.clone());

        require(
            repository.list().await?.is_empty(),
            "a new installation lists readers",
        )?;
        require(
            !repository.grant(Uuid::new_v4()).await?,
            "an unknown account was listed",
        )?;
        require(
            repository.grant(first.user_id).await? && repository.grant(first.user_id).await?,
            "listing an account, or listing it twice, failed",
        )?;
        let listed = repository.list().await?;
        require(
            listed.len() == 1
                && listed[0].user_id == first.user_id
                && listed[0].user_name == first.user_name
                && listed[0].user_email == first.email,
            "the list did not read back the account as stored",
        )?;

        // A service started later, as after a restart, applies what is stored.
        let store: Arc<dyn ReaderStore> = Arc::new(ReaderRepository::new(context.pool.clone()));
        let service = KcnaArchiveService::start_from_environment(store).await;
        require(
            service.admits(first.user_id).await && !service.admits(second.user_id).await,
            "a restarted service did not apply the stored list",
        )?;
        let actor = Uuid::new_v4();
        let both = service.grant_reader(second.user_id, actor).await?;
        require(
            both.len() == 2 && both[0].user_id == second.user_id,
            "the newest grant is not first",
        )?;
        require(
            matches!(
                service.grant_reader(Uuid::new_v4(), actor).await,
                Err(ArchiveError::ReaderNotFound)
            ),
            "an unknown account was not refused through the service",
        )?;

        // A deleted account drops off the list and cannot be listed again.
        let mut connection = context.pool.get().await?;
        diesel::update(users::table.find(second.user_id))
            .set((
                users::user_deleted_at.eq(diesel::dsl::now),
                users::user_purge_after.eq(diesel::dsl::now),
            ))
            .execute(&mut connection)
            .await?;
        drop(connection);
        let active = service.readers().await?;
        require(
            active.len() == 1 && active[0].user_id == first.user_id,
            "a deleted account stayed on the list",
        )?;
        require(
            !service.admits(second.user_id).await,
            "a deleted account is still admitted",
        )?;
        require(
            !repository.grant(second.user_id).await?,
            "a deleted account was listed",
        )?;

        let none = service.revoke_reader(first.user_id, actor).await?;
        require(
            none.is_empty() && !service.admits(first.user_id).await,
            "removal did not reach the database and the service",
        )?;
        repository.revoke(first.user_id).await?;
        require(
            repository.list().await?.is_empty(),
            "removing an absent account changed the list",
        )
    })
}
