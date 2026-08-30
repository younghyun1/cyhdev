//! Maps forum failures to stable HTTP errors.

use crate::{
    errors::code_error::{CodeError, CodeErrorResp, code_err},
    features::{accounts::authorization_error::AuthorizationError, forum::error::ForumError},
};

pub(super) fn map_forum_error(error: ForumError) -> CodeErrorResp {
    let code = match &error {
        ForumError::Pool(_) => CodeError::POOL_ERROR,
        ForumError::Query(_) | ForumError::CountOverflow => CodeError::DB_QUERY_ERROR,
        ForumError::Authorization(AuthorizationError::AccountNotFound) => {
            CodeError::UNAUTHORIZED_ACCESS
        }
        ForumError::Authorization(error) if error.is_retryable() => CodeError::DB_QUERY_ERROR,
        ForumError::Authorization(_) | ForumError::NotOwner | ForumError::ModerationForbidden => {
            CodeError::FORUM_FORBIDDEN
        }
        ForumError::TopicNotFound => CodeError::FORUM_TOPIC_NOT_FOUND,
        ForumError::ReplyNotFound => CodeError::FORUM_REPLY_NOT_FOUND,
        ForumError::NotificationNotFound => CodeError::FORUM_NOTIFICATION_NOT_FOUND,
        ForumError::TopicLocked | ForumError::ContentStateConflict | ForumError::NoChange => {
            CodeError::FORUM_CONTENT_CONFLICT
        }
        ForumError::RevisionConflict => CodeError::FORUM_REVISION_CONFLICT,
        ForumError::SubscriptionSaturated { .. } => CodeError::FORUM_SUBSCRIPTION_SATURATED,
        ForumError::InvalidTitle
        | ForumError::InvalidBody
        | ForumError::InvalidModerationReason
        | ForumError::InvalidSearch
        | ForumError::InvalidPageSize
        | ForumError::InvalidCursor
        | ForumError::InvalidRevision => CodeError::FORUM_INVALID_REQUEST,
        ForumError::WriteThrottled { .. } => CodeError::FORUM_WRITE_THROTTLED,
    };
    let retry_after = match &error {
        ForumError::WriteThrottled {
            retry_after,
            saturated,
        } => {
            tracing::warn!(
                event = "forum_write_rejected",
                capacity_saturated = *saturated,
                retry_after_seconds = retry_after.as_secs(),
                "Forum write rejected"
            );
            Some(*retry_after)
        }
        _ => None,
    };
    let response = code_err(code, error);
    match retry_after {
        Some(retry_after) => response.with_retry_after(retry_after),
        None => response,
    }
}
