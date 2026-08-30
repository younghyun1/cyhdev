//! Errors shared by the account repository and service boundaries.

use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel_async::pooled_connection::bb8::RunError;
use thiserror::Error;
use uuid::Uuid;

/// Failure modes for account use cases.
#[derive(Debug, Error)]
pub enum AccountError {
    #[error("database pool unavailable")]
    Pool(#[source] RunError),
    #[error("account query failed")]
    Query(#[source] DieselError),
    #[error("account mutation failed")]
    Mutation(#[source] DieselError),
    #[error("email address already exists")]
    DuplicateEmail(#[source] DieselError),
    #[error("user name already exists")]
    DuplicateUserName(#[source] DieselError),
    #[error("account was not found")]
    AccountNotFound,
    #[error("credentials were not accepted")]
    InvalidCredentials,
    #[error("the protected system actor cannot be deleted or purged")]
    SystemActorProtected,
    #[error("current database role does not authorize account hard purge")]
    HardPurgeRequesterUnauthorized,
    #[error("media cleanup record was not found")]
    MediaCleanupNotFound,
    #[error("media cleanup object location is invalid")]
    InvalidMediaCleanupLocation,
    #[error("media cleanup original URL did not match")]
    MediaCleanupOriginalUrlMismatch,
    #[error("media cleanup record is already resolved to another object")]
    MediaCleanupAlreadyResolved,
    #[error("media cleanup object is already represented by another record")]
    MediaCleanupObjectConflict,
    #[error("account has already been deleted")]
    AccountAlreadyDeleted,
    #[error("account must be deleted before retained identity can be purged")]
    AccountNotDeleted,
    #[error("account must be hard-purged before profile metadata can be finalized")]
    AccountNotHardPurged,
    #[error("retained identity cannot be purged before {purge_after}")]
    RetentionPeriodActive { purge_after: chrono::DateTime<chrono::Utc> },
    #[error("account credentials changed while deletion was being confirmed")]
    AccountChanged,
    #[error("the protected system actor used for neutral identity defaults is missing")]
    SystemActorMissing,
    #[error("retained account identity is missing")]
    RetainedIdentityMissing,
    #[error("account-retention schedule exceeded the supported timestamp range")]
    RetentionScheduleOverflow,
    #[error("profile-cleanup row count exceeded the supported platform range")]
    ProfileCleanupCountOverflow,
    #[error("email-verification token was not found")]
    EmailVerificationTokenNotFound,
    #[error("password-reset token was not found")]
    PasswordResetTokenNotFound,
    #[error("token has already been consumed")]
    TokenAlreadyConsumed,
    #[error("account email is already verified")]
    EmailAlreadyVerified,
    #[error("role ID {0} is not recognized")]
    InvalidRoleId(Uuid),
    #[error("email address is invalid")]
    InvalidEmail,
    #[error("user name is invalid")]
    InvalidUserName,
    #[error("country, language, or subdivision selection is invalid")]
    InvalidAccountGeography,
    #[error("password does not meet policy")]
    InvalidPassword,
    #[error("password did not match")]
    WrongPassword,
    #[error("password work queue reached its fixed limit of {max_jobs} jobs")]
    PasswordWorkSaturated { max_jobs: usize },
    #[error("password hashing failed")]
    PasswordHash(#[source] anyhow::Error),
    #[error("password verification failed")]
    PasswordVerification(#[source] anyhow::Error),
    #[error("email-verification token has expired")]
    EmailVerificationTokenExpired,
    #[error("email-verification token creation time is in the future")]
    EmailVerificationTokenFabricated,
    #[error("email-verification token has already been used")]
    EmailVerificationTokenAlreadyUsed,
    #[error("password-reset token has expired")]
    PasswordResetTokenExpired,
    #[error("password-reset token creation time is in the future")]
    PasswordResetTokenFabricated,
    #[error("password-reset token has already been used")]
    PasswordResetTokenAlreadyUsed,
    #[error("operating-system entropy was unavailable for session creation")]
    SessionEntropy(#[source] getrandom::Error),
    #[error("session-token generation exhausted collision retries")]
    SessionTokenCollision,
    #[error("session store reached its fixed limit of {max_sessions} sessions")]
    SessionStoreSaturated { max_sessions: usize },
}

impl From<diesel::result::Error> for AccountError {
    fn from(error: diesel::result::Error) -> Self {
        Self::Mutation(error)
    }
}

impl AccountError {
    /// Whether retrying the same operation can succeed without changing its input.
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Pool(_)
            | Self::PasswordWorkSaturated { .. }
            | Self::SessionEntropy(_)
            | Self::SessionTokenCollision
            | Self::SessionStoreSaturated { .. } => true,
            Self::Query(error) | Self::Mutation(error) => is_retryable_diesel_error(error),
            _ => false,
        }
    }
}

fn is_retryable_diesel_error(error: &DieselError) -> bool {
    match error {
        DieselError::DatabaseError(kind, _) => matches!(
            *kind,
            DatabaseErrorKind::SerializationFailure | DatabaseErrorKind::ClosedConnection
        ),
        _ => false,
    }
}
