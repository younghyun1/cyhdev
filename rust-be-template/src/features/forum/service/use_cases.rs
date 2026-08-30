//! Forum use cases and account-authority coordination.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::features::{
    accounts::authorization_error::AuthorizationError,
    forum::{
        domain::{
            enums::ForumModerationAction,
            models::{
                ForumCapabilities, ForumModerationAuditPage, ForumModerationReceipt,
                ForumMutationReceipt, ForumNotificationPage, ForumNotificationPruneReport,
                ForumTopicDetail, ForumTopicPage,
            },
            validation::{
                DEFAULT_AUDIT_PAGE_SIZE, DEFAULT_NOTIFICATION_PAGE_SIZE, DEFAULT_REPLY_PAGE_SIZE,
                DEFAULT_TOPIC_PAGE_SIZE, ForumBody, ForumModerationReason, ForumTitle,
            },
        },
        error::ForumError,
        repository::moderation::{ModerateReplyCommand, ModerateTopicCommand},
        service::{forum_service::ForumService, validation, write_limiter::ForumWriteKind},
    },
};

impl ForumService {
    pub async fn capabilities(
        &self,
        user_id: Option<Uuid>,
    ) -> Result<ForumCapabilities, ForumError> {
        let Some(user_id) = user_id else {
            return Ok(ForumCapabilities {
                authenticated: false,
                can_post: false,
                can_moderate: false,
            });
        };
        match self.accounts.acquire_forum_authority(user_id).await {
            Ok(lease) => {
                let authority = lease.authority();
                Ok(ForumCapabilities {
                    authenticated: true,
                    can_post: true,
                    can_moderate: authority.can_moderate,
                })
            }
            Err(AuthorizationError::AccountNotFound) => Ok(ForumCapabilities {
                authenticated: false,
                can_post: false,
                can_moderate: false,
            }),
            Err(error) => Err(error.into()),
        }
    }

    pub async fn topics(
        &self,
        search: Option<String>,
        pinned: Option<bool>,
        activity: Option<DateTime<Utc>>,
        topic_id: Option<Uuid>,
        limit: Option<u16>,
    ) -> Result<ForumTopicPage, ForumError> {
        let search = validation::search(search)?;
        self.repository
            .topic_page(
                search.as_ref(),
                validation::topic_cursor(pinned, activity, topic_id)?,
                validation::page_size(limit, DEFAULT_TOPIC_PAGE_SIZE)?,
            )
            .await
    }

    pub async fn topic(
        &self,
        topic_id: Uuid,
        viewer: Option<Uuid>,
        reply_created: Option<DateTime<Utc>>,
        reply_id: Option<Uuid>,
        limit: Option<u16>,
    ) -> Result<ForumTopicDetail, ForumError> {
        self.repository
            .topic_detail(
                topic_id,
                viewer,
                validation::reply_cursor(reply_created, reply_id)?,
                validation::page_size(limit, DEFAULT_REPLY_PAGE_SIZE)?,
            )
            .await
    }

    pub async fn create_topic(
        &self,
        user_id: Uuid,
        title: String,
        body: String,
    ) -> Result<ForumMutationReceipt, ForumError> {
        let title = ForumTitle::try_new(title).map_err(|_| ForumError::InvalidTitle)?;
        let body = ForumBody::try_new(body).map_err(|_| ForumError::InvalidBody)?;
        self.write_limiter
            .check(user_id, ForumWriteKind::Topic)
            .await
            .map_err(|rejection| ForumError::WriteThrottled {
                retry_after: rejection.retry_after,
                saturated: rejection.saturated,
            })?;
        let lease = self.accounts.acquire_forum_authority(user_id).await?;
        let result = self
            .repository
            .create_topic(lease.authority().user_id, &title, &body, Utc::now())
            .await;
        drop(lease);
        result
    }

    pub async fn update_topic(
        &self,
        user_id: Uuid,
        topic_id: Uuid,
        title: String,
        body: String,
        expected_revision: i32,
    ) -> Result<ForumMutationReceipt, ForumError> {
        let lease = self.accounts.acquire_forum_authority(user_id).await?;
        let title = ForumTitle::try_new(title).map_err(|_| ForumError::InvalidTitle)?;
        let body = ForumBody::try_new(body).map_err(|_| ForumError::InvalidBody)?;
        let result = self
            .repository
            .update_topic(
                lease.authority().user_id,
                topic_id,
                &title,
                &body,
                validation::revision(expected_revision)?,
                Utc::now(),
            )
            .await;
        drop(lease);
        result
    }

    pub async fn delete_topic(
        &self,
        user_id: Uuid,
        topic_id: Uuid,
        expected_revision: i32,
    ) -> Result<ForumMutationReceipt, ForumError> {
        let lease = self.accounts.acquire_forum_authority(user_id).await?;
        let result = self
            .repository
            .delete_topic(
                lease.authority().user_id,
                topic_id,
                validation::revision(expected_revision)?,
                Utc::now(),
            )
            .await;
        drop(lease);
        result
    }

    pub async fn create_reply(
        &self,
        user_id: Uuid,
        topic_id: Uuid,
        body: String,
    ) -> Result<ForumMutationReceipt, ForumError> {
        let body = ForumBody::try_new(body).map_err(|_| ForumError::InvalidBody)?;
        self.write_limiter
            .check(user_id, ForumWriteKind::Reply)
            .await
            .map_err(|rejection| ForumError::WriteThrottled {
                retry_after: rejection.retry_after,
                saturated: rejection.saturated,
            })?;
        let lease = self.accounts.acquire_forum_authority(user_id).await?;
        let result = self
            .repository
            .create_reply(lease.authority().user_id, topic_id, &body, Utc::now())
            .await;
        drop(lease);
        result
    }

    pub async fn update_reply(
        &self,
        user_id: Uuid,
        reply_id: Uuid,
        body: String,
        expected_revision: i32,
    ) -> Result<ForumMutationReceipt, ForumError> {
        let lease = self.accounts.acquire_forum_authority(user_id).await?;
        let body = ForumBody::try_new(body).map_err(|_| ForumError::InvalidBody)?;
        let result = self
            .repository
            .update_reply(
                lease.authority().user_id,
                reply_id,
                &body,
                validation::revision(expected_revision)?,
                Utc::now(),
            )
            .await;
        drop(lease);
        result
    }

    pub async fn delete_reply(
        &self,
        user_id: Uuid,
        reply_id: Uuid,
        expected_revision: i32,
    ) -> Result<ForumMutationReceipt, ForumError> {
        let lease = self.accounts.acquire_forum_authority(user_id).await?;
        let result = self
            .repository
            .delete_reply(
                lease.authority().user_id,
                reply_id,
                validation::revision(expected_revision)?,
                Utc::now(),
            )
            .await;
        drop(lease);
        result
    }

    pub async fn set_subscription(
        &self,
        user_id: Uuid,
        topic_id: Uuid,
        enabled: bool,
    ) -> Result<bool, ForumError> {
        let lease = self.accounts.acquire_forum_authority(user_id).await?;
        let result = if enabled {
            self.repository
                .subscribe(lease.authority().user_id, topic_id, Utc::now())
                .await
                .map(|_| true)
        } else {
            self.repository
                .unsubscribe(lease.authority().user_id, topic_id)
                .await
                .map(|_| false)
        };
        drop(lease);
        result
    }

    pub async fn moderate_topic(
        &self,
        user_id: Uuid,
        topic_id: Uuid,
        action: ForumModerationAction,
        reason: String,
        expected_revision: i32,
        request_id: Option<Uuid>,
    ) -> Result<ForumModerationReceipt, ForumError> {
        let lease = self.accounts.acquire_forum_authority(user_id).await?;
        if !lease.authority().can_moderate {
            return Err(ForumError::ModerationForbidden);
        }
        let reason = ForumModerationReason::try_new(reason)
            .map_err(|_| ForumError::InvalidModerationReason)?;
        let result = self
            .repository
            .moderate_topic(ModerateTopicCommand {
                actor_user_id: user_id,
                topic_id,
                action,
                reason: &reason,
                expected_revision: validation::revision(expected_revision)?,
                request_id,
                now: Utc::now(),
            })
            .await;
        drop(lease);
        result
    }

    pub async fn moderate_reply(
        &self,
        user_id: Uuid,
        reply_id: Uuid,
        action: ForumModerationAction,
        reason: String,
        expected_revision: i32,
        request_id: Option<Uuid>,
    ) -> Result<ForumModerationReceipt, ForumError> {
        let lease = self.accounts.acquire_forum_authority(user_id).await?;
        if !lease.authority().can_moderate {
            return Err(ForumError::ModerationForbidden);
        }
        let reason = ForumModerationReason::try_new(reason)
            .map_err(|_| ForumError::InvalidModerationReason)?;
        let result = self
            .repository
            .moderate_reply(ModerateReplyCommand {
                actor_user_id: user_id,
                reply_id,
                action,
                reason: &reason,
                expected_revision: validation::revision(expected_revision)?,
                request_id,
                now: Utc::now(),
            })
            .await;
        drop(lease);
        result
    }

    pub async fn notifications(
        &self,
        user_id: Uuid,
        created: Option<DateTime<Utc>>,
        notification_id: Option<Uuid>,
        limit: Option<u16>,
    ) -> Result<ForumNotificationPage, ForumError> {
        let lease = self.accounts.acquire_forum_authority(user_id).await?;
        let result = self
            .repository
            .notification_page(
                lease.authority().user_id,
                validation::timestamp_cursor(created, notification_id)?,
                validation::page_size(limit, DEFAULT_NOTIFICATION_PAGE_SIZE)?,
                Utc::now(),
            )
            .await;
        drop(lease);
        result
    }

    pub async fn mark_notification_read(
        &self,
        user_id: Uuid,
        notification_id: Uuid,
    ) -> Result<DateTime<Utc>, ForumError> {
        let lease = self.accounts.acquire_forum_authority(user_id).await?;
        let result = self
            .repository
            .mark_notification_read(lease.authority().user_id, notification_id, Utc::now())
            .await;
        drop(lease);
        result
    }

    pub async fn moderation_audit(
        &self,
        user_id: Uuid,
        created: Option<DateTime<Utc>>,
        audit_id: Option<Uuid>,
        limit: Option<u16>,
    ) -> Result<ForumModerationAuditPage, ForumError> {
        let lease = self.accounts.acquire_forum_authority(user_id).await?;
        if !lease.authority().can_moderate {
            return Err(ForumError::ModerationForbidden);
        }
        let result = self
            .repository
            .moderation_audit_page(
                validation::timestamp_cursor(created, audit_id)?,
                validation::page_size(limit, DEFAULT_AUDIT_PAGE_SIZE)?,
            )
            .await;
        drop(lease);
        result
    }

    pub async fn prune_notifications(&self) -> Result<ForumNotificationPruneReport, ForumError> {
        self.repository
            .prune_expired_notifications(Utc::now())
            .await
    }
}
