//! Maps account use-case failures onto the existing HTTP error contract.

use crate::{
    errors::code_error::{CodeError, CodeErrorResp, code_err},
    features::accounts::error::AccountError,
};

/// Storage operation used to retain the endpoint-specific legacy error code.
#[derive(Clone, Copy)]
pub(super) enum AccountMutation {
    Insert,
    Update,
}

/// Converts an account boundary error into the public HTTP error envelope.
pub(super) fn map_account_error(error: AccountError, mutation: AccountMutation) -> CodeErrorResp {
    let code = match &error {
        AccountError::Pool(_) => CodeError::POOL_ERROR,
        AccountError::Query(_) | AccountError::InvalidRoleId(_) => CodeError::DB_QUERY_ERROR,
        AccountError::Mutation(_) => match mutation {
            AccountMutation::Insert => CodeError::DB_INSERTION_ERROR,
            AccountMutation::Update => CodeError::DB_UPDATE_ERROR,
        },
        AccountError::DuplicateEmail(_) => CodeError::EMAIL_MUST_BE_UNIQUE,
        AccountError::DuplicateUserName(_) => CodeError::USER_NAME_INVALID,
        AccountError::AccountNotFound => CodeError::USER_NOT_FOUND,
        AccountError::SystemActorProtected => CodeError::SYSTEM_ACTOR_PROTECTED,
        AccountError::HardPurgeRequesterUnauthorized => CodeError::IS_NOT_SUPERUSER,
        AccountError::MediaCleanupNotFound => CodeError::MEDIA_CLEANUP_NOT_FOUND,
        AccountError::InvalidMediaCleanupLocation => CodeError::INVALID_REQUEST,
        AccountError::MediaCleanupOriginalUrlMismatch
        | AccountError::MediaCleanupAlreadyResolved
        | AccountError::MediaCleanupObjectConflict => CodeError::MEDIA_CLEANUP_CONFLICT,
        AccountError::AccountAlreadyDeleted
        | AccountError::AccountNotDeleted
        | AccountError::AccountNotHardPurged
        | AccountError::RetentionPeriodActive { .. }
        | AccountError::AccountChanged => CodeError::ACCOUNT_LIFECYCLE_CONFLICT,
        AccountError::SystemActorMissing
        | AccountError::RetainedIdentityMissing
        | AccountError::RetentionScheduleOverflow
        | AccountError::ProfileCleanupCountOverflow => {
            CodeError::DB_QUERY_ERROR
        }
        AccountError::EmailVerificationTokenNotFound => CodeError::INVALID_EMAIL_VERIFICATION_TOKEN,
        AccountError::PasswordResetTokenNotFound => CodeError::DB_QUERY_ERROR,
        AccountError::TokenAlreadyConsumed => CodeError::INVALID_REQUEST,
        AccountError::EmailAlreadyVerified => CodeError::USER_EMAIL_ALREADY_VERIFIED,
        AccountError::InvalidEmail => CodeError::EMAIL_INVALID,
        AccountError::InvalidUserName => CodeError::USER_NAME_INVALID,
        AccountError::InvalidAccountGeography => CodeError::INVALID_REQUEST,
        AccountError::InvalidPassword => CodeError::PASSWORD_INVALID,
        AccountError::WrongPassword => CodeError::WRONG_PW,
        AccountError::PasswordHash(_) => CodeError::COULD_NOT_HASH_PW,
        AccountError::PasswordVerification(_) => CodeError::COULD_NOT_VERIFY_PW,
        AccountError::EmailVerificationTokenExpired => CodeError::EMAIL_VERIFICATION_TOKEN_EXPIRED,
        AccountError::EmailVerificationTokenFabricated => {
            CodeError::EMAIL_VERIFICATION_TOKEN_FABRICATED
        }
        AccountError::EmailVerificationTokenAlreadyUsed => {
            CodeError::EMAIL_VERIFICATION_TOKEN_ALREADY_USED
        }
        AccountError::PasswordResetTokenExpired => CodeError::PASSWORD_RESET_TOKEN_EXPIRED,
        AccountError::PasswordResetTokenFabricated => CodeError::PASSWORD_RESET_TOKEN_FABRICATED,
        AccountError::PasswordResetTokenAlreadyUsed => CodeError::PASSWORD_RESET_TOKEN_ALREADY_USED,
        AccountError::SessionEntropy(_) | AccountError::SessionTokenCollision => {
            CodeError::SESSION_CREATION_FAILED
        }
        AccountError::SessionStoreSaturated { .. } => CodeError::SESSION_STORE_SATURATED,
    };

    code_err(code, error)
}
