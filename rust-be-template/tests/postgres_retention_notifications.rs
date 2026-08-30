//! PostgreSQL coverage for durable pre-purge notification delivery.

mod support;

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Duration;
use uuid::Uuid;

use rust_be_template::features::accounts::{
    domain::retention_notifications::{
        RetentionNotificationDeliveryItem, RetentionNotificationStage,
    },
    error::AccountError,
    service::retention_notifications::{
        RetentionNotificationDeliveryError, RetentionNotificationSender,
    },
};

use support::{
    database::{DatabaseTestFuture, TestDatabase, TestResult, require, run_database_test},
    fixtures::{VALID_PASSWORD, account_test_context, seed_account},
};

#[derive(Default)]
struct FakeSender {
    failures_remaining: tokio::sync::Mutex<usize>,
    attempts: tokio::sync::Mutex<Vec<(Uuid, RetentionNotificationStage)>>,
}

impl FakeSender {
    fn failing(count: usize) -> Self {
        Self {
            failures_remaining: tokio::sync::Mutex::new(count),
            attempts: tokio::sync::Mutex::new(Vec::new()),
        }
    }

    async fn attempts(&self) -> Vec<(Uuid, RetentionNotificationStage)> {
        self.attempts.lock().await.clone()
    }
}

#[async_trait]
impl RetentionNotificationSender for FakeSender {
    async fn send_retention_notification(
        &self,
        notification: &RetentionNotificationDeliveryItem,
    ) -> Result<(), RetentionNotificationDeliveryError> {
        self.attempts
            .lock()
            .await
            .push((notification.notification_id, notification.stage));
        let mut failures = self.failures_remaining.lock().await;
        if *failures == 0 {
            Ok(())
        } else {
            *failures = failures.saturating_sub(1);
            Err(RetentionNotificationDeliveryError::TransportFailed)
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn staged_delivery_is_idempotent_and_retries_durably() -> TestResult {
    run_database_test(staged_delivery_case).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn claims_exclude_overlap_and_hard_purge_cancels_unsent_rows() -> TestResult {
    run_database_test(claim_and_purge_case).await
}

fn staged_delivery_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let admin = seed_account(&context, "NoticeAdmin").await?;
        context
            .accounts
            .assign_role(
                admin.user_id,
                rust_be_template::features::accounts::domain::role::RoleType::Younghyun,
            )
            .await?;
        let account = seed_account(&context, "NoticeDelivery").await?;
        let deletion = context
            .accounts
            .soft_delete_account(account.user_id, VALID_PASSWORD)
            .await?;
        let first_due = deletion.purge_after - Duration::days(7) + Duration::seconds(1);
        let sender = Arc::new(FakeSender::failing(1));

        let failed = context
            .accounts
            .send_due_retention_notifications_with(sender.as_ref(), first_due)
            .await?;
        require(
            failed.claimed == 1 && failed.failed == 1 && failed.delivered == 0,
            "first staged notification did not persist its delivery failure",
        )?;
        let statuses = context
            .accounts
            .retention_notification_status(admin.user_id, None, 100)
            .await?;
        let seven_day = statuses
            .iter()
            .find(|status| status.stage == RetentionNotificationStage::SevenDaysBeforePurge)
            .ok_or(AccountError::AccountChanged)?;
        require(
            seven_day.notification_id.get_version_num() == 7
                && seven_day.attempt_count == 1
                && seven_day.sent_at.is_none()
                && seven_day.last_error.as_deref() == Some("smtp transport failed")
                && seven_day.next_attempt_at == first_due + Duration::minutes(5),
            "failed notification did not retain bounded exponential retry state",
        )?;

        let early = context
            .accounts
            .send_due_retention_notifications_with(
                sender.as_ref(),
                seven_day.next_attempt_at - Duration::seconds(1),
            )
            .await?;
        require(early.claimed == 0, "notification retried before next-attempt time")?;
        let manual_retry_at = first_due + Duration::minutes(1);
        let retry = context
            .repository
            .retry_retention_notification(
                admin.user_id,
                seven_day.notification_id,
                manual_retry_at,
            )
            .await?;
        require(
            retry.next_attempt_at == manual_retry_at,
            "authorized manual retry did not move the due timestamp",
        )?;
        let retried = context
            .accounts
            .send_due_retention_notifications_with(
                sender.as_ref(),
                manual_retry_at + Duration::seconds(1),
            )
            .await?;
        require(
            retried.delivered == 1 && retried.failed == 0,
            "due retry did not finalize the same notification",
        )?;
        let final_stage = context
            .accounts
            .send_due_retention_notifications_with(
                sender.as_ref(),
                deletion.purge_after - Duration::days(1) + Duration::seconds(1),
            )
            .await?;
        require(
            final_stage.delivered == 1,
            "24-hour notification was not delivered at its independent stage",
        )?;
        let attempts = sender.attempts().await;
        require(
            attempts.len() == 3 && attempts.first() == attempts.get(1),
            "retry did not reuse the durable notification identity",
        )?;

        context
            .repository
            .hard_purge_account(
                admin.user_id,
                account.user_id,
                deletion.purge_after + Duration::seconds(1),
            )
            .await?;
        let retained_status = context
            .accounts
            .retention_notification_status(admin.user_id, None, 100)
            .await?;
        require(
            retained_status.len() == 2
                && retained_status.iter().all(|status| status.sent_at.is_some()),
            "hard purge erased sent notification audit evidence",
        )
    })
}

fn claim_and_purge_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let admin = seed_account(&context, "NoticeClaimAdmin").await?;
        context
            .accounts
            .assign_role(
                admin.user_id,
                rust_be_template::features::accounts::domain::role::RoleType::Younghyun,
            )
            .await?;
        let account = seed_account(&context, "NoticeClaim").await?;
        let deletion = context
            .accounts
            .soft_delete_account(account.user_id, VALID_PASSWORD)
            .await?;
        let due = deletion.purge_after - Duration::days(7) + Duration::seconds(1);
        let first = context
            .repository
            .claim_due_retention_notifications(due)
            .await?;
        let overlapping = context
            .repository
            .claim_due_retention_notifications(due)
            .await?;
        let first_notification = first
            .notifications
            .first()
            .ok_or(AccountError::AccountChanged)?;
        require(
            first.claim_token.get_version_num() == 7
                && first.notifications.len() == 1
                && overlapping.notifications.is_empty(),
            "overlapping worker claimed the same staged notification",
        )?;
        let reclaimed = context
            .repository
            .claim_due_retention_notifications(due + Duration::minutes(11))
            .await?;
        let reclaimed_notification = reclaimed
            .notifications
            .first()
            .ok_or(AccountError::AccountChanged)?;
        require(
            reclaimed.notifications.len() == 1
                && reclaimed_notification.notification_id
                    == first_notification.notification_id
                && reclaimed_notification.attempt_count == 2,
            "expired claim was not reclaimed with durable attempt state",
        )?;

        context
            .repository
            .hard_purge_account(
                admin.user_id,
                account.user_id,
                deletion.purge_after + Duration::seconds(1),
            )
            .await?;
        let ids = first
            .notifications
            .iter()
            .map(|notification| notification.notification_id)
            .collect::<Vec<_>>();
        let deliverable = context
            .repository
            .revalidate_retention_notification_claim(
                reclaimed.claim_token,
                &ids,
                due + Duration::minutes(11),
            )
            .await?;
        require(
            deliverable.is_empty(),
            "notification remained deliverable after retained email was removed",
        )?;
        let statuses = context
            .accounts
            .retention_notification_status(admin.user_id, None, 100)
            .await?;
        require(
            statuses.len() == 2
                && statuses
                    .iter()
                    .all(|status| status.cancelled_at.is_some()),
            "hard purge did not preserve cancelled notification audit rows",
        )
    })
}
