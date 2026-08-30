//! Transactional topic creation, editing, and author deletion.

use chrono::{DateTime, Utc};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    features::forum::{
        domain::{
            models::ForumMutationReceipt,
            validation::{ForumBody, ForumTitle},
        },
        error::ForumError,
        repository::{
            enums::DbForumContentState, forum_repository::ForumRepository,
            records::NewForumTopicRecord, subscriptions::ensure_subscription,
        },
    },
    schema::{forum_topic_subscriptions, forum_topics},
};

const DELETED_TOPIC_TITLE: &str = "[deleted]";
const DELETED_CONTENT_BODY: &str = "[deleted]";

impl ForumRepository {
    pub async fn create_topic(
        &self,
        actor_user_id: Uuid,
        title: &ForumTitle,
        body: &ForumBody,
        now: DateTime<Utc>,
    ) -> Result<ForumMutationReceipt, ForumError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<ForumMutationReceipt, ForumError, _>(async move |connection| {
                let (item_id, revision, updated_at) = diesel::insert_into(forum_topics::table)
                    .values(NewForumTopicRecord {
                        forum_topic_author_user_id: actor_user_id,
                        forum_topic_title: title.as_ref(),
                        forum_topic_body: body.as_ref(),
                        forum_topic_created_at: now,
                        forum_topic_updated_at: now,
                        forum_topic_last_activity_at: now,
                    })
                    .returning((
                        forum_topics::forum_topic_id,
                        forum_topics::forum_topic_revision,
                        forum_topics::forum_topic_updated_at,
                    ))
                    .get_result::<(Uuid, i32, DateTime<Utc>)>(&mut *connection)
                    .await?;
                ensure_subscription(connection, actor_user_id, item_id, now).await?;
                Ok(ForumMutationReceipt {
                    item_id,
                    revision,
                    updated_at,
                })
            })
            .await
    }

    pub async fn update_topic(
        &self,
        actor_user_id: Uuid,
        topic_id: Uuid,
        title: &ForumTitle,
        body: &ForumBody,
        expected_revision: i32,
        now: DateTime<Utc>,
    ) -> Result<ForumMutationReceipt, ForumError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<ForumMutationReceipt, ForumError, _>(async move |connection| {
                let state = lock_topic(connection, topic_id).await?;
                validate_author_mutation(state, actor_user_id, expected_revision)?;
                if state.content_state != DbForumContentState::Visible {
                    return Err(ForumError::ContentStateConflict);
                }
                let (revision, updated_at) = diesel::update(
                    forum_topics::table.filter(forum_topics::forum_topic_id.eq(topic_id)),
                )
                .set((
                    forum_topics::forum_topic_title.eq(title.as_ref()),
                    forum_topics::forum_topic_body.eq(body.as_ref()),
                    forum_topics::forum_topic_revision.eq(forum_topics::forum_topic_revision + 1),
                    forum_topics::forum_topic_updated_at.eq(now),
                    forum_topics::forum_topic_edited_at.eq(now),
                ))
                .returning((
                    forum_topics::forum_topic_revision,
                    forum_topics::forum_topic_updated_at,
                ))
                .get_result::<(i32, DateTime<Utc>)>(&mut *connection)
                .await?;
                Ok(ForumMutationReceipt {
                    item_id: topic_id,
                    revision,
                    updated_at,
                })
            })
            .await
    }

    pub async fn delete_topic(
        &self,
        actor_user_id: Uuid,
        topic_id: Uuid,
        expected_revision: i32,
        now: DateTime<Utc>,
    ) -> Result<ForumMutationReceipt, ForumError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<ForumMutationReceipt, ForumError, _>(async move |connection| {
                let state = lock_topic(connection, topic_id).await?;
                validate_author_mutation(state, actor_user_id, expected_revision)?;
                if state.content_state == DbForumContentState::Deleted {
                    return Err(ForumError::ContentStateConflict);
                }
                let (revision, updated_at) = diesel::update(
                    forum_topics::table.filter(forum_topics::forum_topic_id.eq(topic_id)),
                )
                .set((
                    forum_topics::forum_topic_title.eq(DELETED_TOPIC_TITLE),
                    forum_topics::forum_topic_body.eq(DELETED_CONTENT_BODY),
                    forum_topics::forum_topic_content_state.eq(DbForumContentState::Deleted),
                    forum_topics::forum_topic_is_pinned.eq(false),
                    forum_topics::forum_topic_hidden_at.eq(Option::<DateTime<Utc>>::None),
                    forum_topics::forum_topic_deleted_at.eq(Some(now)),
                    forum_topics::forum_topic_revision.eq(forum_topics::forum_topic_revision + 1),
                    forum_topics::forum_topic_updated_at.eq(now),
                ))
                .returning((
                    forum_topics::forum_topic_revision,
                    forum_topics::forum_topic_updated_at,
                ))
                .get_result::<(i32, DateTime<Utc>)>(&mut *connection)
                .await?;
                diesel::delete(forum_topic_subscriptions::table.filter(
                    forum_topic_subscriptions::forum_topic_subscription_topic_id.eq(topic_id),
                ))
                .execute(&mut *connection)
                .await?;
                Ok(ForumMutationReceipt {
                    item_id: topic_id,
                    revision,
                    updated_at,
                })
            })
            .await
    }
}

#[derive(Clone, Copy)]
pub(super) struct LockedTopic {
    pub(super) author_user_id: Uuid,
    pub(super) content_state: DbForumContentState,
    pub(super) access_state: crate::features::forum::repository::enums::DbForumTopicAccessState,
    pub(super) is_pinned: bool,
    pub(super) hidden_at: Option<DateTime<Utc>>,
    pub(super) revision: i32,
}

pub(super) async fn lock_topic(
    connection: &mut diesel_async::AsyncPgConnection,
    topic_id: Uuid,
) -> Result<LockedTopic, ForumError> {
    forum_topics::table
        .filter(forum_topics::forum_topic_id.eq(topic_id))
        .select((
            forum_topics::forum_topic_author_user_id,
            forum_topics::forum_topic_content_state,
            forum_topics::forum_topic_access_state,
            forum_topics::forum_topic_is_pinned,
            forum_topics::forum_topic_hidden_at,
            forum_topics::forum_topic_revision,
        ))
        .for_update()
        .first::<(
            Uuid,
            DbForumContentState,
            crate::features::forum::repository::enums::DbForumTopicAccessState,
            bool,
            Option<DateTime<Utc>>,
            i32,
        )>(&mut *connection)
        .await
        .optional()?
        .map(|row| LockedTopic {
            author_user_id: row.0,
            content_state: row.1,
            access_state: row.2,
            is_pinned: row.3,
            hidden_at: row.4,
            revision: row.5,
        })
        .ok_or(ForumError::TopicNotFound)
}

fn validate_author_mutation(
    state: LockedTopic,
    actor_user_id: Uuid,
    expected_revision: i32,
) -> Result<(), ForumError> {
    if state.author_user_id != actor_user_id {
        return Err(ForumError::NotOwner);
    }
    if state.revision != expected_revision {
        return Err(ForumError::RevisionConflict);
    }
    Ok(())
}
