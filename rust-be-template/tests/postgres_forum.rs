mod support;

use std::sync::Arc;

use diesel::{Connection, ExpressionMethods, QueryDsl, dsl::count_star, pg::PgConnection};
use diesel_async::RunQueryDsl;
use diesel_migrations::MigrationHarness;
use uuid::Uuid;

use rust_be_template::{
    features::{
        accounts::domain::role::RoleType,
        forum::{
            domain::{enums::ForumModerationAction, validation::ForumBody},
            error::ForumError,
            repository::forum_repository::ForumRepository,
            service::forum_service::ForumService,
        },
    },
    init::db_migrations::MIGRATIONS,
    schema::{
        forum_moderation_audit_events, forum_notifications, forum_replies,
        forum_topic_subscriptions, forum_topics,
    },
};

use support::{
    database::{
        BoxError, DatabaseTestFuture, TestDatabase, TestResult, require, run_database_test,
    },
    fixtures::{VALID_PASSWORD, account_test_context, seed_account},
    migrations::rewind_to_migration,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn forum_enforces_retention_authority_fanout_and_keysets() -> TestResult {
    run_database_test(forum_case).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn forum_down_migration_refuses_existing_content() -> TestResult {
    run_database_test(forum_down_guard_case).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn forum_reply_activity_is_monotonic_and_topic_deletion_cleans_subscriptions() -> TestResult {
    run_database_test(forum_activity_and_cleanup_case).await
}

fn forum_activity_and_cleanup_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let author = seed_account(&context, "ForumActivityAuthor").await?;
        context
            .accounts
            .verify_email(author.verification_token)
            .await?;
        let repository = Arc::new(ForumRepository::new(context.pool.clone()));
        let forum = ForumService::new(Arc::clone(&repository), Arc::clone(&context.accounts));
        let topic = forum
            .create_topic(
                author.user_id,
                "Monotonic reply activity".to_owned(),
                "Opening body".to_owned(),
            )
            .await?;
        let later = chrono::DateTime::<chrono::Utc>::from_timestamp(
            chrono::Utc::now().timestamp() + 7_200,
            0,
        )
        .ok_or(ForumError::CountOverflow)?;
        let earlier = later - chrono::Duration::hours(1);
        let body = ForumBody::try_new("A retained reply".to_owned())
            .map_err(|_| ForumError::InvalidBody)?;
        repository
            .create_reply(author.user_id, topic.item_id, &body, later)
            .await?;
        repository
            .create_reply(author.user_id, topic.item_id, &body, earlier)
            .await?;

        let detail = forum
            .topic(topic.item_id, Some(author.user_id), None, None, Some(10))
            .await?;
        require(
            detail.topic.topic.last_activity_at == later && detail.topic.topic.updated_at == later,
            "an older reply timestamp moved topic activity backward",
        )?;
        repository
            .delete_topic(
                author.user_id,
                topic.item_id,
                detail.topic.topic.revision,
                later + chrono::Duration::hours(1),
            )
            .await?;
        let mut connection = context.pool.get().await?;
        let subscriptions = forum_topic_subscriptions::table
            .filter(forum_topic_subscriptions::forum_topic_subscription_topic_id.eq(topic.item_id))
            .select(count_star())
            .first::<i64>(&mut connection)
            .await?;
        require(
            subscriptions == 0,
            "topic deletion retained unusable subscriptions",
        )
    })
}

fn forum_down_guard_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let author = seed_account(&context, "ForumDownGuard").await?;
        context
            .accounts
            .verify_email(author.verification_token)
            .await?;
        let repository = Arc::new(ForumRepository::new(context.pool.clone()));
        let forum = ForumService::new(repository, Arc::clone(&context.accounts));
        forum
            .create_topic(
                author.user_id,
                "Rollback guard topic".to_owned(),
                "Retained forum content".to_owned(),
            )
            .await?;
        let database_url = database.database_url().to_owned();
        let refused = tokio::task::spawn_blocking(move || -> TestResult<bool> {
            let mut connection = PgConnection::establish(&database_url)?;
            rewind_to_migration(&mut connection, "202608300600000000")?;
            Ok(connection.revert_last_migration(MIGRATIONS).is_err())
        })
        .await??;
        require(refused, "forum down migration accepted existing content")
    })
}

fn forum_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let author = seed_account(&context, "ForumAuthor").await?;
        let subscriber = seed_account(&context, "ForumSubscriber").await?;
        let moderator = seed_account(&context, "ForumModerator").await?;
        for token in [
            author.verification_token,
            subscriber.verification_token,
            moderator.verification_token,
        ] {
            context.accounts.verify_email(token).await?;
        }
        context
            .accounts
            .assign_role(moderator.user_id, RoleType::Younghyun)
            .await?;
        let repository = Arc::new(ForumRepository::new(context.pool.clone()));
        let forum = ForumService::new(Arc::clone(&repository), Arc::clone(&context.accounts));

        let topic = forum
            .create_topic(
                author.user_id,
                "A retained forum topic".to_owned(),
                "Plain text opening body".to_owned(),
            )
            .await?;
        let search_hit = forum
            .topics(Some("retained".to_owned()), None, None, None, Some(10))
            .await?;
        let search_miss = forum
            .topics(
                Some("definitely-no-match".to_owned()),
                None,
                None,
                None,
                Some(10),
            )
            .await?;
        require(
            search_hit
                .items
                .iter()
                .any(|item| item.topic.topic_id == topic.item_id),
            "full-text search missed a visible topic",
        )?;
        require(
            search_miss.items.is_empty(),
            "full-text search returned a nonmatching topic",
        )?;
        require(
            forum
                .set_subscription(subscriber.user_id, topic.item_id, true)
                .await?,
            "subscriber was not subscribed",
        )?;
        let first_reply = forum
            .create_reply(
                author.user_id,
                topic.item_id,
                "First retained reply".to_owned(),
            )
            .await?;
        let second_reply = forum
            .create_reply(
                author.user_id,
                topic.item_id,
                "Second retained reply".to_owned(),
            )
            .await?;

        let tied_at = chrono::Utc::now();
        let mut tie_connection = context.pool.get().await?;
        diesel::update(forum_replies::table.filter(
            forum_replies::forum_reply_id.eq_any([first_reply.item_id, second_reply.item_id]),
        ))
        .set((
            forum_replies::forum_reply_created_at.eq(tied_at),
            forum_replies::forum_reply_updated_at.eq(tied_at),
        ))
        .execute(&mut tie_connection)
        .await?;
        drop(tie_connection);
        let first_page = forum
            .topic(topic.item_id, None, None, None, Some(1))
            .await?;
        let cursor = first_page
            .replies
            .next_cursor
            .ok_or_else(|| Box::new(ForumError::InvalidCursor) as BoxError)?;
        let second_page = forum
            .topic(
                topic.item_id,
                None,
                Some(cursor.created_at),
                Some(cursor.reply_id),
                Some(1),
            )
            .await?;
        require(
            first_page.replies.items[0].reply.reply_id
                != second_page.replies.items[0].reply.reply_id,
            "reply keyset duplicated a tied timestamp row",
        )?;

        let detail = forum
            .topic(topic.item_id, Some(author.user_id), None, None, Some(100))
            .await?;
        require(
            detail.topic.topic.reply_count == 2,
            "retained reply count did not increment transactionally",
        )?;
        require(
            detail.replies.items.len() == 2,
            "reply page did not return both retained rows",
        )?;
        require(
            detail
                .replies
                .items
                .iter()
                .all(|item| !item.author.is_deleted()),
            "repeated author projection was consumed after its first use",
        )?;

        let inbox = forum
            .notifications(subscriber.user_id, None, None, Some(100))
            .await?;
        require(
            inbox.items.len() == 2,
            "reply fanout did not create exactly one notification per reply",
        )?;
        require(
            inbox
                .items
                .iter()
                .all(|item| item.notification.actor_user_id == author.user_id),
            "notification actor attribution changed",
        )?;
        let read_at = forum
            .mark_notification_read(
                subscriber.user_id,
                inbox.items[0].notification.notification_id,
            )
            .await?;
        require(
            read_at <= chrono::Utc::now(),
            "notification read timestamp is in the future",
        )?;

        let forbidden = forum
            .moderate_reply(
                subscriber.user_id,
                first_reply.item_id,
                ForumModerationAction::ReplyHidden,
                "Hide content after a documented moderation review".to_owned(),
                first_reply.revision,
                None,
            )
            .await;
        require(
            matches!(forbidden, Err(ForumError::ModerationForbidden)),
            "ordinary account received forum moderation authority",
        )?;
        let hidden = forum
            .moderate_reply(
                moderator.user_id,
                first_reply.item_id,
                ForumModerationAction::ReplyHidden,
                "Hide content after a documented moderation review".to_owned(),
                first_reply.revision,
                Some(Uuid::now_v7()),
            )
            .await?;
        let restored = forum
            .moderate_reply(
                moderator.user_id,
                first_reply.item_id,
                ForumModerationAction::ReplyRestored,
                "Restore content after completing moderation review".to_owned(),
                hidden.revision,
                None,
            )
            .await?;
        require(
            restored.revision == hidden.revision + 1,
            "moderation revision did not advance",
        )?;
        let mut audit_connection = context.pool.get().await?;
        let audit_update = diesel::update(
            forum_moderation_audit_events::table.filter(
                forum_moderation_audit_events::forum_moderation_audit_event_id
                    .eq(hidden.audit_event_id),
            ),
        )
        .set(
            forum_moderation_audit_events::forum_moderation_audit_event_reason
                .eq("Attempt to rewrite immutable audit history"),
        )
        .execute(&mut audit_connection)
        .await;
        let audit_delete = diesel::delete(
            forum_moderation_audit_events::table.filter(
                forum_moderation_audit_events::forum_moderation_audit_event_id
                    .eq(hidden.audit_event_id),
            ),
        )
        .execute(&mut audit_connection)
        .await;
        require(
            audit_update.is_err() && audit_delete.is_err(),
            "append-only forum audit accepted mutation",
        )?;
        drop(audit_connection);
        let stale = forum
            .update_reply(
                author.user_id,
                second_reply.item_id,
                "stale update".to_owned(),
                second_reply.revision + 1,
            )
            .await;
        require(
            matches!(stale, Err(ForumError::RevisionConflict)),
            "stale reply edit did not conflict",
        )?;

        let expired_created_at = chrono::Utc::now() - chrono::Duration::days(91);
        let mut expiry_connection = context.pool.get().await?;
        diesel::update(forum_notifications::table.filter(
            forum_notifications::forum_notification_recipient_user_id.eq(subscriber.user_id),
        ))
        .set((
            forum_notifications::forum_notification_created_at.eq(expired_created_at),
            forum_notifications::forum_notification_expires_at
                .eq(expired_created_at + chrono::Duration::days(90)),
        ))
        .execute(&mut expiry_connection)
        .await?;
        drop(expiry_connection);
        let pruned = forum.prune_notifications().await?;
        require(
            pruned.deleted == 2 && !pruned.remaining_expired,
            "bounded notification expiry did not remove the due inbox",
        )?;
        forum
            .create_reply(
                author.user_id,
                topic.item_id,
                "Notification retained until recipient deletion".to_owned(),
            )
            .await?;
        let recreated_inbox = forum
            .notifications(subscriber.user_id, None, None, Some(100))
            .await?;
        require(
            recreated_inbox.items.len() == 1,
            "notification inbox was not repopulated before lifecycle cleanup",
        )?;

        context
            .accounts
            .soft_delete_account(author.user_id, VALID_PASSWORD)
            .await?;
        let retained = forum
            .topic(topic.item_id, None, None, None, Some(100))
            .await?;
        require(
            retained.topic.author.is_deleted(),
            "deleted topic author was not masked",
        )?;
        require(
            retained
                .replies
                .items
                .iter()
                .all(|item| item.author.is_deleted()),
            "deleted repeated reply author was not masked",
        )?;

        context
            .accounts
            .soft_delete_account(subscriber.user_id, VALID_PASSWORD)
            .await?;
        let mut connection = context.pool.get().await?;
        let subscriptions = forum_topic_subscriptions::table
            .filter(
                forum_topic_subscriptions::forum_topic_subscription_user_id.eq(subscriber.user_id),
            )
            .select(count_star())
            .first::<i64>(&mut connection)
            .await?;
        let notifications = forum_notifications::table
            .filter(
                forum_notifications::forum_notification_recipient_user_id.eq(subscriber.user_id),
            )
            .select(count_star())
            .first::<i64>(&mut connection)
            .await?;
        let retained_topics = forum_topics::table
            .filter(forum_topics::forum_topic_id.eq(topic.item_id))
            .select(count_star())
            .first::<i64>(&mut connection)
            .await?;
        require(
            subscriptions == 0 && notifications == 0,
            "soft deletion retained private forum inbox state",
        )?;
        require(
            retained_topics == 1,
            "soft deletion removed authored forum content",
        )
    })
}
