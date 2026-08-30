//! Bounded delivery orchestration for durable account-retention notices.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures_util::{StreamExt, stream};
use lettre::AsyncTransport;

use crate::{
    features::accounts::{
        domain::retention_notifications::{
            RETENTION_NOTIFICATION_DELIVERY_CONCURRENCY, RETENTION_NOTIFICATION_ERROR_CHARS,
            RETENTION_NOTIFICATION_STATUS_LIMIT, RetentionNotificationCursor,
            RetentionNotificationDeliveryItem, RetentionNotificationRetryReceipt,
            RetentionNotificationRunReport, RetentionNotificationStatus,
            retention_notification_retry_delay,
        },
        error::AccountError,
        service::account_service::AccountService,
    },
    util::email::retention_notification::AccountRetentionNotificationEmail,
};

/// Identity-free classification stored for an unsuccessful delivery attempt.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RetentionNotificationDeliveryError {
    SmtpAdmissionSaturated,
    MessageBuildFailed,
    TransportFailed,
}

impl RetentionNotificationDeliveryError {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SmtpAdmissionSaturated => "smtp admission saturated",
            Self::MessageBuildFailed => "notification message build failed",
            Self::TransportFailed => "smtp transport failed",
        }
    }
}

#[async_trait]
pub trait RetentionNotificationSender: Send + Sync {
    async fn send_retention_notification(
        &self,
        notification: &RetentionNotificationDeliveryItem,
    ) -> Result<(), RetentionNotificationDeliveryError>;
}

#[async_trait]
impl RetentionNotificationSender for AccountService {
    async fn send_retention_notification(
        &self,
        notification: &RetentionNotificationDeliveryItem,
    ) -> Result<(), RetentionNotificationDeliveryError> {
        let email_job = self
            .email_jobs
            .clone()
            .try_acquire_owned()
            .map_err(|_| RetentionNotificationDeliveryError::SmtpAdmissionSaturated)?;
        let message = AccountRetentionNotificationEmail::new(
            notification.notification_id,
            notification.stage,
            notification.purge_after,
        )
        .to_message(&notification.retained_email)
        .map_err(|_| RetentionNotificationDeliveryError::MessageBuildFailed)?;
        let result = self
            .email_client
            .send(message)
            .await
            .map_err(|_| RetentionNotificationDeliveryError::TransportFailed);
        drop(email_job);
        result.map(|_| ())
    }
}

impl AccountService {
    /// Runs one fixed-size due batch with the production bounded SMTP sender.
    pub async fn send_due_retention_notifications(
        &self,
    ) -> Result<RetentionNotificationRunReport, AccountError> {
        self.send_due_retention_notifications_with(self, Utc::now())
            .await
    }

    /// Testable clock and sender boundary; never sends unless the caller supplies a sender.
    pub async fn send_due_retention_notifications_with<S>(
        &self,
        sender: &S,
        now: DateTime<Utc>,
    ) -> Result<RetentionNotificationRunReport, AccountError>
    where
        S: RetentionNotificationSender + ?Sized,
    {
        let _run = self.retention_notification_run_gate.lock().await;
        let batch = self
            .repository
            .claim_due_retention_notifications(now)
            .await?;
        let mut report = RetentionNotificationRunReport {
            claimed: batch.notifications.len(),
            ..RetentionNotificationRunReport::default()
        };
        if batch.notifications.is_empty() {
            return Ok(report);
        }

        let delivery_guard = self.retention_notification_delivery_gate.read().await;
        let notification_ids = batch
            .notifications
            .iter()
            .map(|notification| notification.notification_id)
            .collect::<Vec<_>>();
        let deliveries = self
            .repository
            .revalidate_retention_notification_claim(batch.claim_token, &notification_ids, now)
            .await?;
        report.skipped = report.claimed.saturating_sub(deliveries.len());
        let claim_token = batch.claim_token;

        let outcomes = stream::iter(deliveries.into_iter().map(|notification| async move {
            let delivery = sender.send_retention_notification(&notification).await;
            let persisted = match delivery {
                Ok(()) => self
                    .repository
                    .mark_retention_notification_sent(
                        notification.notification_id,
                        claim_token,
                        now,
                    )
                    .await
                    .map(|updated| (updated, true)),
                Err(error) => {
                    let next_attempt_at = now
                        .checked_add_signed(retention_notification_retry_delay(
                            notification.attempt_count,
                        ))
                        .ok_or(AccountError::RetentionScheduleOverflow)?;
                    let bounded_error = bounded_delivery_error(error);
                    self.repository
                        .mark_retention_notification_failed(
                            notification.notification_id,
                            claim_token,
                            now,
                            next_attempt_at,
                            bounded_error,
                        )
                        .await
                        .map(|updated| (updated, false))
                }
            }?;
            Ok::<(bool, bool), AccountError>(persisted)
        }))
        .buffer_unordered(RETENTION_NOTIFICATION_DELIVERY_CONCURRENCY)
        .collect::<Vec<_>>()
        .await;
        drop(delivery_guard);

        for outcome in outcomes {
            let (persisted, delivered) = outcome?;
            if !persisted {
                report.skipped = report.skipped.saturating_add(1);
            } else if delivered {
                report.delivered = report.delivered.saturating_add(1);
            } else {
                report.failed = report.failed.saturating_add(1);
            }
        }
        Ok(report)
    }

    pub async fn retention_notification_status(
        &self,
        requester_id: uuid::Uuid,
        cursor: Option<RetentionNotificationCursor>,
        requested_limit: i64,
    ) -> Result<Vec<RetentionNotificationStatus>, AccountError> {
        let limit = requested_limit.clamp(1, RETENTION_NOTIFICATION_STATUS_LIMIT);
        self.repository
            .retention_notification_status(requester_id, cursor, limit)
            .await
    }

    pub async fn retry_retention_notification(
        &self,
        requester_id: uuid::Uuid,
        notification_id: uuid::Uuid,
    ) -> Result<RetentionNotificationRetryReceipt, AccountError> {
        let _run = self.retention_notification_run_gate.lock().await;
        let _delivery = self.retention_notification_delivery_gate.write().await;
        self.repository
            .retry_retention_notification(requester_id, notification_id, Utc::now())
            .await
    }
}

fn bounded_delivery_error(error: RetentionNotificationDeliveryError) -> &'static str {
    let error = error.as_str();
    if error.chars().count() <= RETENTION_NOTIFICATION_ERROR_CHARS {
        error
    } else {
        "retention notification delivery failed"
    }
}

#[cfg(test)]
mod tests {
    use super::{RetentionNotificationDeliveryError, bounded_delivery_error};

    #[test]
    fn persisted_delivery_errors_are_identity_free_and_bounded() {
        for error in [
            RetentionNotificationDeliveryError::SmtpAdmissionSaturated,
            RetentionNotificationDeliveryError::MessageBuildFailed,
            RetentionNotificationDeliveryError::TransportFailed,
        ] {
            let persisted = bounded_delivery_error(error);
            assert!(persisted.len() <= 512);
            assert!(!persisted.contains('@'));
        }
    }
}
