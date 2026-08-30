//! Scheduled account-maintenance use cases.

use crate::features::accounts::{error::AccountError, service::account_service::AccountService};

impl AccountService {
    pub async fn purge_unverified_accounts(&self) -> Result<usize, AccountError> {
        let _session_consistency = self.session_consistency.write().await;
        let user_ids = self
            .repository
            .purge_unverified_accounts(chrono::Utc::now())
            .await?;
        for user_id in &user_ids {
            self.sessions.remove_for_user(*user_id).await;
        }
        Ok(user_ids.len())
    }
}
