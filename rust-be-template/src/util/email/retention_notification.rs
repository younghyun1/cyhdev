//! Deterministic account-retention notification messages.

use chrono::{DateTime, Utc};
use lettre::message::Mailbox;
use uuid::Uuid;

use crate::{
    DOMAIN_NAME,
    features::accounts::domain::retention_notifications::RetentionNotificationStage,
};

const TEMPLATE: &str = include_str!("account_retention_notification.html");

pub struct AccountRetentionNotificationEmail {
    body: String,
    message_id: String,
    subject: &'static str,
}

impl AccountRetentionNotificationEmail {
    pub fn new(
        notification_id: Uuid,
        stage: RetentionNotificationStage,
        purge_after: DateTime<Utc>,
    ) -> Self {
        let (remaining, subject) = match stage {
            RetentionNotificationStage::SevenDaysBeforePurge => (
                "7 days",
                "Account data retention: 7 days remaining",
            ),
            RetentionNotificationStage::OneDayBeforePurge => (
                "24 hours",
                "Account data retention: 24 hours remaining",
            ),
        };
        Self {
            body: TEMPLATE
                .replace("$1", remaining)
                .replace("$2", &purge_after.to_rfc3339()),
            message_id: format!(
                "<account-retention-{}@{DOMAIN_NAME}>",
                notification_id.simple()
            ),
            subject,
        }
    }

    pub fn to_message(self, retained_email: &str) -> anyhow::Result<lettre::Message> {
        let from = format!("cyhdev.com <donotreply@{DOMAIN_NAME}>")
            .parse::<Mailbox>()?;
        let to = retained_email.parse::<Mailbox>()?;
        Ok(lettre::Message::builder()
            .from(from)
            .to(to)
            .subject(self.subject)
            .message_id(Some(self.message_id))
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(self.body)?)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    use super::AccountRetentionNotificationEmail;
    use crate::features::accounts::domain::retention_notifications::RetentionNotificationStage;

    #[test]
    fn message_has_stable_id_and_no_account_identity() -> anyhow::Result<()> {
        let notification_id = Uuid::now_v7();
        let purge_after = Utc
            .with_ymd_and_hms(2026, 9, 30, 12, 0, 0)
            .single()
            .ok_or_else(|| anyhow::anyhow!("fixed UTC test timestamp was invalid"))?;
        let message = AccountRetentionNotificationEmail::new(
            notification_id,
            RetentionNotificationStage::SevenDaysBeforePurge,
            purge_after,
        )
        .to_message("retained@example.test")?;
        let formatted = String::from_utf8(message.formatted())?;

        assert!(formatted.contains(&format!(
            "Message-ID: <account-retention-{}@cyhdev.com>",
            notification_id.simple()
        )));
        assert!(formatted.contains("7 days"));
        assert!(!formatted.contains("user name"));
        Ok(())
    }
}
