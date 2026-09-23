//! Throttling and session revocation around authenticated current-password confirmations.
//!
//! Profile updates, account deletion, and OIDC link changes all re-prove the password. A
//! stolen session cookie must not turn them into an unthrottled password oracle, so each
//! attempt is charged to the source IP before Argon2 work, and wrong passwords are charged
//! to the account. Exhausting the account budget revokes the session presenting it.

use std::{net::IpAddr, sync::Arc};

use uuid::Uuid;

use crate::{
    errors::code_error::{CodeError, CodeErrorResp, code_err},
    features::accounts::{
        api::auth_abuse::map_auth_throttle_rejection,
        domain::auth_abuse::{AuthEndpoint, AuthIdentity, FailureBudget},
        error::AccountError,
    },
    init::state::ServerState,
};

/// Source and account of one confirmation attempt.
#[derive(Clone, Copy)]
pub(super) struct Confirmation {
    pub(super) user_id: Uuid,
    pub(super) client_ip: IpAddr,
}

impl Confirmation {
    /// Charges the attempt to the source and refuses an account with no failures left.
    pub(super) async fn admit(&self, state: &ServerState) -> Result<(), CodeErrorResp> {
        let abuse = state.auth_abuse_service();
        abuse
            .check_ip(AuthEndpoint::PasswordConfirmation, self.client_ip)
            .await
            .map_err(map_auth_throttle_rejection)?;
        abuse
            .ensure_failure_budget(
                AuthEndpoint::PasswordConfirmation,
                AuthIdentity::Account(self.user_id),
            )
            .await
            .map_err(map_auth_throttle_rejection)
    }

    /// Settles the account budget and maps a wrong password to its public response.
    ///
    /// Other errors pass through `map_other` unchanged.
    pub(super) async fn settle<T>(
        &self,
        state: &Arc<ServerState>,
        session_token: Option<&str>,
        result: Result<T, AccountError>,
        map_other: fn(AccountError) -> CodeErrorResp,
    ) -> Result<T, CodeErrorResp> {
        let abuse = state.auth_abuse_service();
        let identity = AuthIdentity::Account(self.user_id);
        match result {
            Ok(value) => {
                abuse
                    .forget_identity(AuthEndpoint::PasswordConfirmation, identity)
                    .await;
                Ok(value)
            }
            Err(AccountError::WrongPassword) => {
                let budget = abuse
                    .record_failure(AuthEndpoint::PasswordConfirmation, identity)
                    .await;
                match budget {
                    Ok(FailureBudget::Remaining) => Err(code_err(
                        CodeError::ACCOUNT_PASSWORD_CONFIRMATION_FAILED,
                        AccountError::WrongPassword,
                    )),
                    // A saturated failure table fails closed like an exhausted budget.
                    Ok(FailureBudget::Exhausted) | Err(_) => {
                        Err(self.revoke_session(state, session_token).await)
                    }
                }
            }
            Err(error) => Err(map_other(error)),
        }
    }

    async fn revoke_session(
        &self,
        state: &Arc<ServerState>,
        session_token: Option<&str>,
    ) -> CodeErrorResp {
        let revoked = match session_token {
            Some(session_token) => state.account_service().logout(session_token).await,
            None => false,
        };
        tracing::warn!(
            event = "password_confirmation_session_revoked",
            user_id = %self.user_id,
            revoked,
            "Revoked session after repeated wrong password confirmations"
        );
        code_err(
            CodeError::PASSWORD_CONFIRMATION_SESSION_REVOKED,
            "password confirmation budget exhausted",
        )
    }
}
