//! Failures specific to audited role and permission administration.

use diesel::result::{DatabaseErrorKind, Error as DieselError};
use thiserror::Error;
use uuid::Uuid;

use super::error::AccountError;

#[derive(Debug, Error)]
pub enum AuthorizationError {
    #[error("account repository was unavailable")]
    AccountRepository(#[from] AccountError),
    #[error("authorization query failed")]
    Query(#[from] DieselError),
    #[error("current database role does not authorize role administration")]
    Unauthorized,
    #[error("authorization account was not found")]
    AccountNotFound,
    #[error("protected system actor cannot receive a login role")]
    SystemActorProtected,
    #[error("role ID {0} is not recognized")]
    InvalidRoleId(Uuid),
    #[error("permission was not found")]
    PermissionNotFound,
    #[error("stored permission name is invalid")]
    InvalidPermissionName,
    #[error("authorization audit event refers to a missing permanent user tombstone")]
    AuditUserMissing,
    #[error("authorization reason must contain 8 to 500 characters")]
    InvalidReason,
    #[error("authorization search must contain at most 100 characters")]
    InvalidSearch,
    #[error("authorization page size must be between 1 and 100")]
    InvalidPageSize,
    #[error("the last active Younghyun role cannot be removed")]
    LastActiveYounghyun,
    #[error("an administrator cannot remove their own Younghyun role")]
    SelfLockout,
    #[error("Younghyun permissions cannot be revoked")]
    YounghyunPermissionProtected,
    #[error("the requested authority state is already current")]
    NoChange,
}

impl AuthorizationError {
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::AccountRepository(error) => error.is_retryable(),
            Self::Query(DieselError::DatabaseError(kind, _)) => matches!(
                *kind,
                DatabaseErrorKind::SerializationFailure | DatabaseErrorKind::ClosedConnection
            ),
            _ => false,
        }
    }
}
