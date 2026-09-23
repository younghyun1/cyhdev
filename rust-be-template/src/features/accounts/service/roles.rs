//! Account-role mutation use cases.

use std::sync::Arc;

use uuid::Uuid;

use crate::features::accounts::{
    domain::role::RoleType,
    error::AccountError,
    service::{account_service::AccountService, session_coordination::run_to_completion},
};

impl AccountService {
    /// Assigns the account's exclusive role, then refreshes or revokes its sessions.
    pub async fn assign_role(
        self: &Arc<Self>,
        user_id: Uuid,
        role_type: RoleType,
    ) -> Result<RoleType, AccountError> {
        let service = Arc::clone(self);
        run_to_completion(async move {
            let _authority_consistency = service.authority_consistency.write().await;
            let _session_consistency = service.session_consistency.write().await;
            service.repository.assign_role(user_id, role_type).await?;
            service
                .refresh_sessions_after_commit(user_id, "assign_role")
                .await;
            Ok(role_type)
        })
        .await
    }
}
