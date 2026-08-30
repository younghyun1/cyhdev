//! Bounded moderation-audit reads.

use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

use crate::{
    features::{
        accounts::domain::public_author::PublicAuthor,
        forum::{domain::{models::{ForumModerationAuditPage, ForumModerationAuditView, ForumTimestampCursor}, validation::ForumPageSize},
            error::ForumError, repository::{forum_repository::ForumRepository, records::{ForumAuditRow, audit_from_row}}},
    },
    persistence::public_authors::load_public_authors,
    schema::forum_moderation_audit_events,
};

impl ForumRepository {
    pub async fn moderation_audit_page(
        &self, before: Option<ForumTimestampCursor>, page_size: ForumPageSize,
    ) -> Result<ForumModerationAuditPage, ForumError> {
        let mut connection = self.connection().await?;
        let mut query = forum_moderation_audit_events::table
            .select((forum_moderation_audit_events::forum_moderation_audit_event_id,
                forum_moderation_audit_events::forum_moderation_audit_event_actor_user_id,
                forum_moderation_audit_events::forum_moderation_audit_event_topic_id,
                forum_moderation_audit_events::forum_moderation_audit_event_reply_id,
                forum_moderation_audit_events::forum_moderation_audit_event_action,
                forum_moderation_audit_events::forum_moderation_audit_event_reason,
                forum_moderation_audit_events::forum_moderation_audit_event_request_id,
                forum_moderation_audit_events::forum_moderation_audit_event_created_at))
            .order((forum_moderation_audit_events::forum_moderation_audit_event_created_at.desc(),
                forum_moderation_audit_events::forum_moderation_audit_event_id.desc())).into_boxed();
        if let Some(cursor) = before {
            query = query.filter(forum_moderation_audit_events::forum_moderation_audit_event_created_at.lt(cursor.created_at)
                .or(forum_moderation_audit_events::forum_moderation_audit_event_created_at.eq(cursor.created_at)
                    .and(forum_moderation_audit_events::forum_moderation_audit_event_id.lt(cursor.item_id))));
        }
        let mut rows = query.limit(i64::from(page_size.into_inner()) + 1).load::<ForumAuditRow>(&mut connection).await?;
        let next_cursor = if rows.len() > usize::from(page_size.into_inner()) {
            rows.pop(); rows.last().map(|row| ForumTimestampCursor { created_at: row.7, item_id: row.0 })
        } else { None };
        let events = rows.into_iter().map(audit_from_row).collect::<Vec<_>>();
        let actor_ids = events.iter().map(|event| event.actor_user_id).collect::<Vec<_>>();
        let authors = load_public_authors(&mut connection, &actor_ids).await?;
        let items = events.into_iter().map(|event| {
            let actor = authors.get(&event.actor_user_id).cloned().unwrap_or_else(PublicAuthor::deleted);
            ForumModerationAuditView { event, actor }
        }).collect();
        Ok(ForumModerationAuditPage { items, next_cursor })
    }
}
