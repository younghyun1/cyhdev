//! Diesel records private to forum persistence.

use chrono::{DateTime, Utc};
use diesel::{Insertable, Queryable, Selectable};
use uuid::Uuid;

use crate::{
    features::forum::{
        domain::models::{ForumModerationAuditEvent, ForumNotification, ForumReply, ForumTopic},
        repository::enums::{
            DbForumContentState, DbForumModerationAction, DbForumNotificationKind,
            DbForumTopicAccessState,
        },
    },
    schema::{
        forum_moderation_audit_events, forum_replies, forum_topic_subscriptions, forum_topics,
    },
};

#[derive(Queryable, Selectable)]
#[diesel(table_name = forum_topics)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(super) struct ForumTopicRecord {
    pub(super) forum_topic_id: Uuid,
    pub(super) forum_topic_author_user_id: Uuid,
    pub(super) forum_topic_title: String,
    pub(super) forum_topic_body: String,
    pub(super) forum_topic_content_state: DbForumContentState,
    pub(super) forum_topic_access_state: DbForumTopicAccessState,
    pub(super) forum_topic_is_pinned: bool,
    pub(super) forum_topic_revision: i32,
    pub(super) forum_topic_reply_count: i64,
    pub(super) forum_topic_created_at: DateTime<Utc>,
    pub(super) forum_topic_updated_at: DateTime<Utc>,
    pub(super) forum_topic_last_activity_at: DateTime<Utc>,
    pub(super) forum_topic_edited_at: Option<DateTime<Utc>>,
}

impl From<ForumTopicRecord> for ForumTopic {
    fn from(record: ForumTopicRecord) -> Self {
        let visible = record.forum_topic_content_state == DbForumContentState::Visible;
        Self {
            topic_id: record.forum_topic_id,
            author_user_id: record.forum_topic_author_user_id,
            title: visible.then_some(record.forum_topic_title),
            body: visible.then_some(record.forum_topic_body),
            content_state: record.forum_topic_content_state.into(),
            access_state: record.forum_topic_access_state.into(),
            is_pinned: record.forum_topic_is_pinned,
            revision: record.forum_topic_revision,
            reply_count: record.forum_topic_reply_count,
            created_at: record.forum_topic_created_at,
            updated_at: record.forum_topic_updated_at,
            last_activity_at: record.forum_topic_last_activity_at,
            edited_at: record.forum_topic_edited_at,
        }
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = forum_replies)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(super) struct ForumReplyRecord {
    pub(super) forum_reply_id: Uuid,
    pub(super) forum_reply_topic_id: Uuid,
    pub(super) forum_reply_author_user_id: Uuid,
    pub(super) forum_reply_body: String,
    pub(super) forum_reply_content_state: DbForumContentState,
    pub(super) forum_reply_revision: i32,
    pub(super) forum_reply_created_at: DateTime<Utc>,
    pub(super) forum_reply_updated_at: DateTime<Utc>,
    pub(super) forum_reply_edited_at: Option<DateTime<Utc>>,
}

impl From<ForumReplyRecord> for ForumReply {
    fn from(record: ForumReplyRecord) -> Self {
        let visible = record.forum_reply_content_state == DbForumContentState::Visible;
        Self {
            reply_id: record.forum_reply_id,
            topic_id: record.forum_reply_topic_id,
            author_user_id: record.forum_reply_author_user_id,
            body: visible.then_some(record.forum_reply_body),
            content_state: record.forum_reply_content_state.into(),
            revision: record.forum_reply_revision,
            created_at: record.forum_reply_created_at,
            updated_at: record.forum_reply_updated_at,
            edited_at: record.forum_reply_edited_at,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = forum_topics)]
pub(super) struct NewForumTopicRecord<'a> {
    pub(super) forum_topic_author_user_id: Uuid,
    pub(super) forum_topic_title: &'a str,
    pub(super) forum_topic_body: &'a str,
    pub(super) forum_topic_created_at: DateTime<Utc>,
    pub(super) forum_topic_updated_at: DateTime<Utc>,
    pub(super) forum_topic_last_activity_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = forum_replies)]
pub(super) struct NewForumReplyRecord<'a> {
    pub(super) forum_reply_topic_id: Uuid,
    pub(super) forum_reply_author_user_id: Uuid,
    pub(super) forum_reply_body: &'a str,
    pub(super) forum_reply_created_at: DateTime<Utc>,
    pub(super) forum_reply_updated_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = forum_topic_subscriptions)]
pub(super) struct NewForumSubscriptionRecord {
    pub(super) forum_topic_subscription_topic_id: Uuid,
    pub(super) forum_topic_subscription_user_id: Uuid,
    pub(super) forum_topic_subscription_created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = forum_moderation_audit_events)]
pub(super) struct NewForumModerationAuditRecord<'a> {
    pub(super) forum_moderation_audit_event_actor_user_id: Uuid,
    pub(super) forum_moderation_audit_event_topic_id: Option<Uuid>,
    pub(super) forum_moderation_audit_event_reply_id: Option<Uuid>,
    pub(super) forum_moderation_audit_event_action: DbForumModerationAction,
    pub(super) forum_moderation_audit_event_reason: &'a str,
    pub(super) forum_moderation_audit_event_request_id: Option<Uuid>,
    pub(super) forum_moderation_audit_event_created_at: DateTime<Utc>,
}

pub(super) type ForumNotificationRow = (
    Uuid,
    Uuid,
    Uuid,
    Uuid,
    Uuid,
    DbForumNotificationKind,
    DateTime<Utc>,
    DateTime<Utc>,
    Option<DateTime<Utc>>,
    String,
    DbForumContentState,
);

pub(super) fn notification_from_row(row: ForumNotificationRow) -> ForumNotification {
    ForumNotification {
        notification_id: row.0,
        recipient_user_id: row.1,
        actor_user_id: row.2,
        topic_id: row.3,
        reply_id: row.4,
        kind: row.5.into(),
        created_at: row.6,
        expires_at: row.7,
        read_at: row.8,
        topic_title: (row.10 == DbForumContentState::Visible).then_some(row.9),
    }
}

pub(super) type ForumAuditRow = (
    Uuid,
    Uuid,
    Option<Uuid>,
    Option<Uuid>,
    DbForumModerationAction,
    String,
    Option<Uuid>,
    DateTime<Utc>,
);

pub(super) fn audit_from_row(row: ForumAuditRow) -> ForumModerationAuditEvent {
    ForumModerationAuditEvent {
        audit_event_id: row.0,
        actor_user_id: row.1,
        topic_id: row.2,
        reply_id: row.3,
        action: row.4.into(),
        reason: row.5,
        request_id: row.6,
        created_at: row.7,
    }
}
