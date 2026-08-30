//! Forum aggregates returned by repository and service boundaries.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::features::accounts::domain::public_author::PublicAuthor;

use super::enums::{
    ForumContentState, ForumModerationAction, ForumNotificationKind,
    ForumTopicAccessState,
};

pub struct ForumTopic {
    pub topic_id: Uuid,
    pub author_user_id: Uuid,
    pub title: Option<String>,
    pub body: Option<String>,
    pub content_state: ForumContentState,
    pub access_state: ForumTopicAccessState,
    pub is_pinned: bool,
    pub revision: i32,
    pub reply_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_activity_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
}

pub struct ForumReply {
    pub reply_id: Uuid,
    pub topic_id: Uuid,
    pub author_user_id: Uuid,
    pub body: Option<String>,
    pub content_state: ForumContentState,
    pub revision: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub edited_at: Option<DateTime<Utc>>,
}

pub struct ForumTopicView {
    pub topic: ForumTopic,
    pub author: PublicAuthor,
}

pub struct ForumReplyView {
    pub reply: ForumReply,
    pub author: PublicAuthor,
}

#[derive(Clone, Copy)]
pub struct ForumTopicCursor {
    pub is_pinned: bool,
    pub last_activity_at: DateTime<Utc>,
    pub topic_id: Uuid,
}

#[derive(Clone, Copy)]
pub struct ForumReplyCursor {
    pub created_at: DateTime<Utc>,
    pub reply_id: Uuid,
}

#[derive(Clone, Copy)]
pub struct ForumTimestampCursor {
    pub created_at: DateTime<Utc>,
    pub item_id: Uuid,
}

pub struct ForumTopicPage {
    pub items: Vec<ForumTopicView>,
    pub next_cursor: Option<ForumTopicCursor>,
}

pub struct ForumReplyPage {
    pub items: Vec<ForumReplyView>,
    pub next_cursor: Option<ForumReplyCursor>,
}

pub struct ForumTopicDetail {
    pub topic: ForumTopicView,
    pub replies: ForumReplyPage,
    pub is_subscribed: bool,
}

pub struct ForumMutationReceipt {
    pub item_id: Uuid,
    pub revision: i32,
    pub updated_at: DateTime<Utc>,
}

pub struct ForumModerationReceipt {
    pub audit_event_id: Uuid,
    pub target_id: Uuid,
    pub revision: i32,
    pub action: ForumModerationAction,
    pub moderated_at: DateTime<Utc>,
}

pub struct ForumNotification {
    pub notification_id: Uuid,
    pub recipient_user_id: Uuid,
    pub actor_user_id: Uuid,
    pub topic_id: Uuid,
    pub reply_id: Uuid,
    pub kind: ForumNotificationKind,
    pub topic_title: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
}

pub struct ForumNotificationView {
    pub notification: ForumNotification,
    pub actor: PublicAuthor,
}

pub struct ForumNotificationPage {
    pub items: Vec<ForumNotificationView>,
    pub next_cursor: Option<ForumTimestampCursor>,
}

pub struct ForumModerationAuditEvent {
    pub audit_event_id: Uuid,
    pub actor_user_id: Uuid,
    pub topic_id: Option<Uuid>,
    pub reply_id: Option<Uuid>,
    pub action: ForumModerationAction,
    pub reason: String,
    pub request_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

pub struct ForumModerationAuditView {
    pub event: ForumModerationAuditEvent,
    pub actor: PublicAuthor,
}

pub struct ForumModerationAuditPage {
    pub items: Vec<ForumModerationAuditView>,
    pub next_cursor: Option<ForumTimestampCursor>,
}

#[derive(Clone, Copy)]
pub struct ForumCapabilities {
    pub authenticated: bool,
    pub can_post: bool,
    pub can_moderate: bool,
}

pub struct ForumNotificationPruneReport {
    pub deleted: usize,
    pub remaining_expired: bool,
}
