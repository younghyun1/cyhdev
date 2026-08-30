use chrono::{DateTime, Utc};
use serde_derive::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::features::forum::domain::{
    enums::ForumModerationAction,
    models::{ForumModerationAuditView, ForumModerationReceipt, ForumTimestampCursor},
};

use super::common::ForumAuthorResponse;

#[derive(Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ForumModerationActionResponse {
    TopicHidden,
    TopicRestored,
    TopicLocked,
    TopicUnlocked,
    TopicPinned,
    TopicUnpinned,
    ReplyHidden,
    ReplyRestored,
}

impl From<ForumModerationAction> for ForumModerationActionResponse {
    fn from(value: ForumModerationAction) -> Self {
        match value {
            ForumModerationAction::TopicHidden => Self::TopicHidden,
            ForumModerationAction::TopicRestored => Self::TopicRestored,
            ForumModerationAction::TopicLocked => Self::TopicLocked,
            ForumModerationAction::TopicUnlocked => Self::TopicUnlocked,
            ForumModerationAction::TopicPinned => Self::TopicPinned,
            ForumModerationAction::TopicUnpinned => Self::TopicUnpinned,
            ForumModerationAction::ReplyHidden => Self::ReplyHidden,
            ForumModerationAction::ReplyRestored => Self::ReplyRestored,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct ForumModerationResponse {
    pub audit_event_id: Uuid,
    pub target_id: Uuid,
    pub revision: i32,
    pub action: ForumModerationActionResponse,
    pub moderated_at: DateTime<Utc>,
}
impl From<ForumModerationReceipt> for ForumModerationResponse {
    fn from(value: ForumModerationReceipt) -> Self {
        Self {
            audit_event_id: value.audit_event_id,
            target_id: value.target_id,
            revision: value.revision,
            action: value.action.into(),
            moderated_at: value.moderated_at,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct ForumModerationAuditItem {
    pub audit_event_id: Uuid,
    pub actor: ForumAuthorResponse,
    #[schema(required)]
    pub topic_id: Option<Uuid>,
    #[schema(required)]
    pub reply_id: Option<Uuid>,
    pub action: ForumModerationActionResponse,
    pub reason: String,
    #[schema(required)]
    pub request_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}
impl From<ForumModerationAuditView> for ForumModerationAuditItem {
    fn from(view: ForumModerationAuditView) -> Self {
        let event = view.event;
        Self {
            audit_event_id: event.audit_event_id,
            actor: view.actor.into(),
            topic_id: event.topic_id,
            reply_id: event.reply_id,
            action: event.action.into(),
            reason: event.reason,
            request_id: event.request_id,
            created_at: event.created_at,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct ForumModerationAuditCursorResponse {
    pub before_created_at: DateTime<Utc>,
    pub before_audit_id: Uuid,
}
impl From<ForumTimestampCursor> for ForumModerationAuditCursorResponse {
    fn from(value: ForumTimestampCursor) -> Self {
        Self {
            before_created_at: value.created_at,
            before_audit_id: value.item_id,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct ForumModerationAuditListResponse {
    pub events: Vec<ForumModerationAuditItem>,
    #[schema(required)]
    pub next_cursor: Option<ForumModerationAuditCursorResponse>,
}
