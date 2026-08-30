//! Forum repository and use-case failures.

use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel_async::pooled_connection::bb8::RunError;
use thiserror::Error;

use crate::features::accounts::authorization_error::AuthorizationError;

#[derive(Debug, Error)]
pub enum ForumError {
    #[error("forum database pool unavailable")]
    Pool(#[source] RunError),
    #[error("forum database query failed")]
    Query(#[from] DieselError),
    #[error("forum account authority lookup failed")]
    Authorization(#[from] AuthorizationError),
    #[error("forum topic was not found")]
    TopicNotFound,
    #[error("forum reply was not found")]
    ReplyNotFound,
    #[error("forum notification was not found")]
    NotificationNotFound,
    #[error("forum content is owned by another account")]
    NotOwner,
    #[error("current account does not have forum moderation permission")]
    ModerationForbidden,
    #[error("forum topic is locked")]
    TopicLocked,
    #[error("forum content state conflicts with this operation")]
    ContentStateConflict,
    #[error("forum content revision changed")]
    RevisionConflict,
    #[error("forum title violates character or byte limits")]
    InvalidTitle,
    #[error("forum body violates character or byte limits")]
    InvalidBody,
    #[error("forum moderation reason must contain 8-500 characters")]
    InvalidModerationReason,
    #[error("forum search violates query limits")]
    InvalidSearch,
    #[error("forum page size must be between 1 and 100")]
    InvalidPageSize,
    #[error("forum cursor fields must be supplied together")]
    InvalidCursor,
    #[error("forum revision must be positive")]
    InvalidRevision,
    #[error("forum state already matches the requested mutation")]
    NoChange,
    #[error("forum topic reached its fixed subscription limit of {maximum}")]
    SubscriptionSaturated { maximum: i64 },
    #[error("forum count exceeded the supported platform range")]
    CountOverflow,
    #[error("forum write budget exhausted")]
    WriteThrottled {
        retry_after: std::time::Duration,
        saturated: bool,
    },
}

impl ForumError {
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Pool(_) => true,
            Self::Query(DieselError::DatabaseError(kind, _)) => matches!(
                *kind,
                DatabaseErrorKind::SerializationFailure | DatabaseErrorKind::ClosedConnection
            ),
            Self::Authorization(error) => error.is_retryable(),
            _ => false,
        }
    }
}
