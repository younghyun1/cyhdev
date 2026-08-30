//! Post-commit session refresh and fail-closed revocation policy.

use tracing::{error, trace};
use uuid::Uuid;

use crate::features::accounts::{
    domain::role::RoleType, error::AccountError, service::account_service::AccountService,
};

impl AccountService {
    pub(super) async fn refresh_sessions_after_commit(
        &self,
        user_id: Uuid,
        mutation: &'static str,
    ) {
        match self.session_snapshot(user_id).await {
            Ok((account, role_type)) => {
                let refreshed = self
                    .sessions
                    .refresh_for_user(user_id, &account, role_type)
                    .await;
                trace!(%user_id, mutation, refreshed, "Refreshed sessions after account mutation");
            }
            Err(error) => {
                let revoked = self.sessions.remove_for_user(user_id).await;
                error!(
                    %user_id,
                    mutation,
                    revoked,
                    retryable = error.is_retryable(),
                    error = %error,
                    "Revoked sessions after post-commit refresh failed"
                );
            }
        }
    }

    async fn session_snapshot(
        &self,
        user_id: Uuid,
    ) -> Result<
        (
            crate::features::accounts::domain::account::SessionAccount,
            RoleType,
        ),
        AccountError,
    > {
        let account = match self.repository.session_account(user_id).await? {
            Some(account) => account,
            None => return Err(AccountError::AccountNotFound),
        };
        let role_type = self
            .repository
            .role_for_user_or_insert_default(user_id, RoleType::User)
            .await?;
        Ok((account, role_type))
    }
}
