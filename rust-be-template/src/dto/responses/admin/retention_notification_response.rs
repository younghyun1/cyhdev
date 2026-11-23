use jiff::Timestamp;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::features::accounts::domain::retention_notifications::RetentionNotificationStage;

#[derive(serde_derive::Serialize, ToSchema)]
pub struct RetentionNotificationStatusItem {
    pub notification_id: Uuid,
    pub user_id: Uuid,
    pub stage: RetentionNotificationStage,
    pub scheduled_for: Timestamp,
    pub next_attempt_at: Timestamp,
    pub attempt_count: i32,
    pub claim_expires_at: Option<Timestamp>,
    pub sent_at: Option<Timestamp>,
    pub cancelled_at: Option<Timestamp>,
    pub last_error: Option<String>,
}

#[derive(serde_derive::Serialize, ToSchema)]
pub struct RetentionNotificationStatusResponse {
    pub notifications: Vec<RetentionNotificationStatusItem>,
    pub next_after_next_attempt_at: Option<Timestamp>,
    pub next_after_notification_id: Option<Uuid>,
}

#[derive(serde_derive::Serialize, ToSchema)]
pub struct RetryRetentionNotificationResponse {
    pub notification_id: Uuid,
    pub next_attempt_at: Timestamp,
}
