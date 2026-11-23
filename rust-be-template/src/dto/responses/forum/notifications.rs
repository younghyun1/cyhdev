use jiff::Timestamp;
use serde_derive::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::features::forum::domain::models::{ForumNotificationView, ForumTimestampCursor};

use super::common::{ForumAuthorResponse, ForumNotificationKindResponse};

#[derive(Serialize, ToSchema)]
pub struct ForumNotificationResponse {
    pub notification_id: Uuid,
    pub actor: ForumAuthorResponse,
    pub topic_id: Uuid,
    pub reply_id: Uuid,
    pub kind: ForumNotificationKindResponse,
    #[schema(required)]
    pub topic_title: Option<String>,
    pub created_at: Timestamp,
    pub expires_at: Timestamp,
    #[schema(required)]
    pub read_at: Option<Timestamp>,
}

impl From<ForumNotificationView> for ForumNotificationResponse {
    fn from(view: ForumNotificationView) -> Self {
        let notification = view.notification;
        Self {
            notification_id: notification.notification_id,
            actor: view.actor.into(),
            topic_id: notification.topic_id,
            reply_id: notification.reply_id,
            kind: notification.kind.into(),
            topic_title: notification.topic_title,
            created_at: notification.created_at,
            expires_at: notification.expires_at,
            read_at: notification.read_at,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct ForumNotificationCursorResponse {
    pub before_created_at: Timestamp,
    pub before_notification_id: Uuid,
}
impl From<ForumTimestampCursor> for ForumNotificationCursorResponse {
    fn from(value: ForumTimestampCursor) -> Self {
        Self {
            before_created_at: value.created_at,
            before_notification_id: value.item_id,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct ForumNotificationListResponse {
    pub notifications: Vec<ForumNotificationResponse>,
    #[schema(required)]
    pub next_cursor: Option<ForumNotificationCursorResponse>,
}

#[derive(Serialize, ToSchema)]
pub struct ForumNotificationReadResponse {
    pub notification_id: Uuid,
    pub read_at: Timestamp,
}
