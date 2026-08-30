//! PostgreSQL coverage for bounded profile-picture history and durable cleanup.

mod support;

use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

use rust_be_template::{
    schema::{media_object_cleanup, user_profile_pictures},
    util::{media::object_store::ObjectLocation, s3::AWS_S3_BUCKET_NAME},
};

use support::{
    database::{DatabaseTestFuture, TestDatabase, TestResult, require, run_database_test},
    fixtures::{account_test_context, seed_account},
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn history_is_capped_and_mutations_preserve_one_active_picture() -> TestResult {
    run_database_test(profile_picture_history_case).await
}

fn profile_picture_history_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let account = seed_account(&context, "ProfileHistory").await?;
        let mut inserted_ids = Vec::new();
        for sequence in 0_u8..9 {
            let location = ObjectLocation::new(
                AWS_S3_BUCKET_NAME,
                format!("profiles/history-{sequence}.avif"),
            );
            let url = location.public_s3_url("us-west-1");
            let replacement = context
                .repository
                .replace_profile_picture(account.user_id, 4, true, Some(&url))
                .await?;
            inserted_ids.push(replacement.profile_picture_id);
        }

        let history = context
            .repository
            .profile_picture_history(account.user_id)
            .await?;
        require(history.len() == 8, "profile-picture history exceeded its cap")?;
        let newest_id = match inserted_ids.last() {
            Some(newest_id) => *newest_id,
            None => return require(false, "profile-picture fixture inserted no rows"),
        };
        let selected_id = match inserted_ids.get(3) {
            Some(selected_id) => *selected_id,
            None => return require(false, "profile-picture fixture was incomplete"),
        };
        let oldest_id = match inserted_ids.first() {
            Some(oldest_id) => *oldest_id,
            None => return require(false, "profile-picture fixture inserted no rows"),
        };
        let newest_is_active = match history.first() {
            Some(picture) => picture.profile_picture_id == newest_id && picture.is_active,
            None => false,
        };
        require(
            history.iter().filter(|picture| picture.is_active).count() == 1
                && newest_is_active,
            "newest uploaded profile picture was not uniquely active",
        )?;

        let selected = context
            .repository
            .select_profile_picture(account.user_id, selected_id)
            .await?;
        let selected = match selected {
            Some(selected) => selected,
            None => return require(false, "owned history entry was not selectable"),
        };
        require(selected.is_active, "selected profile picture was not activated")?;

        let deleted = context
            .repository
            .delete_profile_picture(account.user_id, selected_id)
            .await?;
        let deleted = match deleted {
            Some(deleted) => deleted,
            None => return require(false, "active history entry was not deletable"),
        };
        require(
            deleted.active_profile_picture_id.is_some(),
            "deleting the active profile picture did not select a fallback",
        )?;

        let mut connection = context.pool.get().await?;
        let metadata_count = user_profile_pictures::table
            .filter(user_profile_pictures::user_id.eq(account.user_id))
            .count()
            .get_result::<i64>(&mut connection)
            .await?;
        let active_count = user_profile_pictures::table
            .filter(user_profile_pictures::user_id.eq(account.user_id))
            .filter(user_profile_pictures::user_profile_picture_is_active.eq(true))
            .count()
            .get_result::<i64>(&mut connection)
            .await?;
        let cleanup_count = media_object_cleanup::table
            .filter(
                media_object_cleanup::media_object_cleanup_source_id
                    .eq_any([oldest_id, selected_id]),
            )
            .count()
            .get_result::<i64>(&mut connection)
            .await?;
        drop(connection);
        require(metadata_count == 7, "history metadata count was incorrect after delete")?;
        require(active_count == 1, "history mutation did not preserve one active row")?;
        require(cleanup_count == 2, "retired profile objects were not durably enqueued")
    })
}
