//! Public error mapping for authorization administration.

use crate::{
    errors::code_error::{CodeError, CodeErrorResp, code_err},
    features::accounts::{
        api::account_error::{AccountMutation, map_account_error},
        authorization_error::AuthorizationError,
    },
};

pub(crate) fn map_authorization_error(error: AuthorizationError) -> CodeErrorResp {
    match error {
        AuthorizationError::AccountRepository(error) => {
            map_account_error(error, AccountMutation::Update)
        }
        error => {
            let code = match &error {
                AuthorizationError::Unauthorized => CodeError::IS_NOT_SUPERUSER,
                AuthorizationError::AccountNotFound
                | AuthorizationError::InvalidRoleId(_)
                | AuthorizationError::PermissionNotFound => {
                    CodeError::AUTHORIZATION_RESOURCE_NOT_FOUND
                }
                AuthorizationError::SystemActorProtected => CodeError::SYSTEM_ACTOR_PROTECTED,
                AuthorizationError::LastActiveYounghyun
                | AuthorizationError::SelfLockout
                | AuthorizationError::YounghyunPermissionProtected
                | AuthorizationError::NoChange => CodeError::AUTHORIZATION_CONFLICT,
                AuthorizationError::InvalidReason
                | AuthorizationError::InvalidSearch
                | AuthorizationError::InvalidPageSize => CodeError::INVALID_REQUEST,
                AuthorizationError::Query(_)
                | AuthorizationError::InvalidPermissionName
                | AuthorizationError::AuditUserMissing => CodeError::AUTHORIZATION_DATA_INTEGRITY,
                AuthorizationError::AccountRepository(_) => CodeError::DB_QUERY_ERROR,
            };
            code_err(code, error)
        }
    }
}
