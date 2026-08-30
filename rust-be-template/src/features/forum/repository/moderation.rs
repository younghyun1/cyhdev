//! Permission-gated forum moderation with append-only audit.

use chrono::{DateTime, Utc};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    features::forum::{
        domain::{enums::ForumModerationAction, models::ForumModerationReceipt, validation::ForumModerationReason},
        error::ForumError,
        repository::{enums::{DbForumContentState, DbForumModerationAction, DbForumTopicAccessState}, forum_repository::ForumRepository,
            records::NewForumModerationAuditRecord, replies::lock_reply, topics::lock_topic},
    },
    schema::{forum_moderation_audit_events, forum_replies, forum_topics},
};

pub struct ModerateTopicCommand<'a> {
    pub actor_user_id: Uuid,
    pub topic_id: Uuid,
    pub action: ForumModerationAction,
    pub reason: &'a ForumModerationReason,
    pub expected_revision: i32,
    pub request_id: Option<Uuid>,
    pub now: DateTime<Utc>,
}

pub struct ModerateReplyCommand<'a> {
    pub actor_user_id: Uuid,
    pub reply_id: Uuid,
    pub action: ForumModerationAction,
    pub reason: &'a ForumModerationReason,
    pub expected_revision: i32,
    pub request_id: Option<Uuid>,
    pub now: DateTime<Utc>,
}

impl ForumRepository {
    pub async fn moderate_topic(
        &self,
        command: ModerateTopicCommand<'_>,
    ) -> Result<ForumModerationReceipt, ForumError> {
        let ModerateTopicCommand {
            actor_user_id, topic_id, action, reason, expected_revision, request_id, now,
        } = command;
        let mut connection = self.connection().await?;
        connection.transaction::<ForumModerationReceipt, ForumError, _>(async move |connection| {
            let state = lock_topic(connection, topic_id).await?;
            if state.revision != expected_revision { return Err(ForumError::RevisionConflict); }
            if state.content_state == DbForumContentState::Deleted { return Err(ForumError::ContentStateConflict); }
            let mut content_state = state.content_state;
            let mut access_state = state.access_state;
            let mut is_pinned = state.is_pinned;
            let mut hidden_at = state.hidden_at;
            match action {
                ForumModerationAction::TopicHidden if content_state == DbForumContentState::Visible => { content_state = DbForumContentState::Hidden; hidden_at = Some(now); }
                ForumModerationAction::TopicRestored if content_state == DbForumContentState::Hidden => { content_state = DbForumContentState::Visible; hidden_at = None; }
                ForumModerationAction::TopicLocked if access_state == DbForumTopicAccessState::Open => access_state = DbForumTopicAccessState::Locked,
                ForumModerationAction::TopicUnlocked if access_state == DbForumTopicAccessState::Locked => access_state = DbForumTopicAccessState::Open,
                ForumModerationAction::TopicPinned if !is_pinned => is_pinned = true,
                ForumModerationAction::TopicUnpinned if is_pinned => is_pinned = false,
                ForumModerationAction::ReplyHidden | ForumModerationAction::ReplyRestored => return Err(ForumError::ContentStateConflict),
                _ => return Err(ForumError::NoChange),
            }
            let revision = diesel::update(forum_topics::table.filter(forum_topics::forum_topic_id.eq(topic_id)))
                .set((forum_topics::forum_topic_content_state.eq(content_state), forum_topics::forum_topic_access_state.eq(access_state),
                    forum_topics::forum_topic_is_pinned.eq(is_pinned), forum_topics::forum_topic_hidden_at.eq(hidden_at),
                    forum_topics::forum_topic_revision.eq(forum_topics::forum_topic_revision + 1), forum_topics::forum_topic_updated_at.eq(now)))
                .returning(forum_topics::forum_topic_revision).get_result::<i32>(&mut *connection).await?;
            let audit_event_id = insert_audit(connection, NewForumModerationAuditRecord {
                forum_moderation_audit_event_actor_user_id: actor_user_id,
                forum_moderation_audit_event_topic_id: Some(topic_id),
                forum_moderation_audit_event_reply_id: None,
                forum_moderation_audit_event_action: DbForumModerationAction::from(action),
                forum_moderation_audit_event_reason: reason.as_ref(),
                forum_moderation_audit_event_request_id: request_id,
                forum_moderation_audit_event_created_at: now,
            }).await?;
            Ok(ForumModerationReceipt { audit_event_id, target_id: topic_id, revision, action, moderated_at: now })
        }).await
    }

    pub async fn moderate_reply(
        &self,
        command: ModerateReplyCommand<'_>,
    ) -> Result<ForumModerationReceipt, ForumError> {
        let ModerateReplyCommand {
            actor_user_id, reply_id, action, reason, expected_revision, request_id, now,
        } = command;
        let mut connection = self.connection().await?;
        connection.transaction::<ForumModerationReceipt, ForumError, _>(async move |connection| {
            let record = lock_reply(connection, reply_id).await?;
            if record.forum_reply_revision != expected_revision { return Err(ForumError::RevisionConflict); }
            if record.forum_reply_content_state == DbForumContentState::Deleted { return Err(ForumError::ContentStateConflict); }
            let (content_state, hidden_at) = match action {
                ForumModerationAction::ReplyHidden if record.forum_reply_content_state == DbForumContentState::Visible => (DbForumContentState::Hidden, Some(now)),
                ForumModerationAction::ReplyRestored if record.forum_reply_content_state == DbForumContentState::Hidden => (DbForumContentState::Visible, None),
                ForumModerationAction::TopicHidden | ForumModerationAction::TopicRestored | ForumModerationAction::TopicLocked |
                ForumModerationAction::TopicUnlocked | ForumModerationAction::TopicPinned | ForumModerationAction::TopicUnpinned => return Err(ForumError::ContentStateConflict),
                _ => return Err(ForumError::NoChange),
            };
            let revision = diesel::update(forum_replies::table.filter(forum_replies::forum_reply_id.eq(reply_id)))
                .set((forum_replies::forum_reply_content_state.eq(content_state), forum_replies::forum_reply_hidden_at.eq(hidden_at),
                    forum_replies::forum_reply_revision.eq(forum_replies::forum_reply_revision + 1), forum_replies::forum_reply_updated_at.eq(now)))
                .returning(forum_replies::forum_reply_revision).get_result::<i32>(&mut *connection).await?;
            let audit_event_id = insert_audit(connection, NewForumModerationAuditRecord {
                forum_moderation_audit_event_actor_user_id: actor_user_id,
                forum_moderation_audit_event_topic_id: None,
                forum_moderation_audit_event_reply_id: Some(reply_id),
                forum_moderation_audit_event_action: DbForumModerationAction::from(action),
                forum_moderation_audit_event_reason: reason.as_ref(),
                forum_moderation_audit_event_request_id: request_id,
                forum_moderation_audit_event_created_at: now,
            }).await?;
            Ok(ForumModerationReceipt { audit_event_id, target_id: reply_id, revision, action, moderated_at: now })
        }).await
    }
}

async fn insert_audit(
    connection: &mut diesel_async::AsyncPgConnection,
    record: NewForumModerationAuditRecord<'_>,
) -> Result<Uuid, ForumError> {
    diesel::insert_into(forum_moderation_audit_events::table)
        .values(record)
        .returning(forum_moderation_audit_events::forum_moderation_audit_event_id)
        .get_result(&mut *connection)
        .await
        .map_err(ForumError::Query)
}
