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
        AccountError::InvalidCredentials => CodeError::INVALID_CREDENTIALS,
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
        AccountError::PasswordWorkSaturated { .. } => CodeError::AUTH_THROTTLED,
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
        AccountError::OidcDisabled => CodeError::OIDC_DISABLED,
        AccountError::OidcFlowEntropy(_)
        | AccountError::OidcFlowStoreSaturated { .. }
        | AccountError::OidcTokenExchange(_) => CodeError::OIDC_TEMPORARILY_UNAVAILABLE,
        AccountError::OidcFlowRejected
        | AccountError::OidcTokenValidation(_)
        | AccountError::OidcProviderEmailRejected => CodeError::OIDC_FLOW_REJECTED,
        AccountError::OidcIdentityNotLinked | AccountError::OidcIdentityNotFound => {
            CodeError::OIDC_IDENTITY_NOT_LINKED
        }
        AccountError::OidcIdentityConflict(_)
        | AccountError::OidcProviderAlreadyLinked
        | AccountError::OidcLinkSessionMismatch
        | AccountError::OidcAnotherLoginRequired => CodeError::OIDC_IDENTITY_CONFLICT,
    };

    let response = code_err(code, &error);
    match error {
        AccountError::PasswordWorkSaturated { .. } => {
            tracing::warn!(
                event = "auth_password_work_rejected",
                max_jobs = crate::features::accounts::service::account_service::MAX_PASSWORD_JOBS,
                "Authentication password work rejected"
            );
            response.with_retry_after(std::time::Duration::from_secs(1))
        }
        _ => response,
    }
}

pub(super) fn map_login_error(error: AccountError) -> CodeErrorResp {
    match error {
        AccountError::InvalidCredentials => code_err(
            CodeError::INVALID_CREDENTIALS,
            "credentials were not accepted",
        ),
        error => map_account_error(error, AccountMutation::Update),
    }
}

pub(super) fn map_signup_error(error: AccountError) -> CodeErrorResp {
    match error {
        AccountError::DuplicateEmail(_) | AccountError::DuplicateUserName(_) => code_err(
            CodeError::ACCOUNT_IDENTITY_UNAVAILABLE,
            "requested account identity unavailable",
        ),
        error => map_account_error(error, AccountMutation::Insert),
    }
}

pub(super) fn map_password_reset_error(error: AccountError) -> CodeErrorResp {
    match error {
        AccountError::PasswordResetTokenNotFound
        | AccountError::PasswordResetTokenExpired
        | AccountError::PasswordResetTokenFabricated
        | AccountError::PasswordResetTokenAlreadyUsed
        | AccountError::TokenAlreadyConsumed => code_err(
            CodeError::PASSWORD_RESET_REJECTED,
            "password reset capability was not accepted",
        ),
        error => map_account_error(error, AccountMutation::Update),
    }
}
