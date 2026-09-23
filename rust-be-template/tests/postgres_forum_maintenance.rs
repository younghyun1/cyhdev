//! Forum list summaries and batched notification expiry.

mod support;

use std::sync::Arc;

use diesel::sql_types::Uuid as SqlUuid;
use diesel_async::RunQueryDsl;

use rust_be_template::features::forum::{
    repository::forum_repository::ForumRepository, service::forum_service::ForumService,
};

use support::{
    content::verified_account,
    database::{DatabaseTestFuture, TestDatabase, TestResult, require, run_database_test},
    fixtures::account_test_context,
};

/// More than one 512-row cleanup batch, so a single batch cannot finish.
const EXPIRED_NOTIFICATIONS: i64 = 1_200;

#[tokio::test]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn topic_lists_truncate_bodies_and_pruning_drains_every_batch() -> TestResult {
    run_database_test(maintenance_case).await
}

fn maintenance_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let author = verified_account(&context, "ForumMaintainer", false).await?;
        let recipient = verified_account(&context, "ForumRecipient", false).await?;
        let repository = Arc::new(ForumRepository::new(context.pool.clone()));
        let forum = ForumService::new(repository, Arc::clone(&context.accounts));
        let body = "ü".repeat(1_000);
        let topic = forum
            .create_topic(author.user_id, "Summary fixture".to_owned(), body)
            .await?;

        let page = forum.topics(None, None, None, None, Some(10)).await?;
        let summary = page
            .items
            .iter()
            .find(|view| view.topic.topic_id == topic.item_id)
            .and_then(|view| view.topic.body.as_deref());
        require(
            summary.is_some_and(|body| body.chars().count() == 301),
            "topic list did not truncate the body to 301 characters",
        )?;

        let mut connection = context.pool.get().await?;
        diesel::sql_query(
            "INSERT INTO forum_replies (forum_reply_topic_id, forum_reply_author_user_id, forum_reply_body, forum_reply_created_at, forum_reply_updated_at) \
             SELECT $1, $2, 'Expired fixture reply', now() - INTERVAL '100 days', now() - INTERVAL '100 days' \
             FROM generate_series(1, $3::bigint)",
        )
        .bind::<SqlUuid, _>(topic.item_id)
        .bind::<SqlUuid, _>(author.user_id)
        .bind::<diesel::sql_types::BigInt, _>(EXPIRED_NOTIFICATIONS)
        .execute(&mut connection)
        .await?;
        let inserted = diesel::sql_query(
            "INSERT INTO forum_notifications (forum_notification_recipient_user_id, forum_notification_actor_user_id, forum_notification_topic_id, forum_notification_reply_id, forum_notification_kind, forum_notification_created_at, forum_notification_expires_at) \
             SELECT $3, $2, $1, forum_reply_id, 'topic_reply', forum_reply_created_at, forum_reply_created_at + INTERVAL '90 days' \
             FROM forum_replies WHERE forum_reply_topic_id = $1 AND forum_reply_author_user_id = $2",
        )
        .bind::<SqlUuid, _>(topic.item_id)
        .bind::<SqlUuid, _>(author.user_id)
        .bind::<SqlUuid, _>(recipient.user_id)
        .execute(&mut connection)
        .await?;
        drop(connection);
        require(
            i64::try_from(inserted)? == EXPIRED_NOTIFICATIONS,
            "expired notification fixtures were not inserted",
        )?;

        let report = forum.prune_notifications().await?;
        require(
            i64::try_from(report.deleted)? == EXPIRED_NOTIFICATIONS && !report.remaining_expired,
            "one pruning run did not drain every expired batch",
        )
    })
}
