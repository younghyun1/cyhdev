//! Account-role mutation use cases.

use uuid::Uuid;

use crate::features::accounts::{
    domain::role::RoleType, error::AccountError, service::account_service::AccountService,
};

impl AccountService {
    /// Assigns the account's exclusive role, then refreshes or revokes its sessions.
    pub async fn assign_role(
        &self,
        user_id: Uuid,
        role_type: RoleType,
    ) -> Result<RoleType, AccountError> {
        let _session_consistency = self.session_consistency.write().await;
        self.repository.assign_role(user_id, role_type).await?;
        self.refresh_sessions_after_commit(user_id, "assign_role")
            .await;
        Ok(role_type)
    }
}
