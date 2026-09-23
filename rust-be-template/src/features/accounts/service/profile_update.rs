//! Full-profile update use case with password confirmation and session refresh.

use chrono::Utc;
use uuid::Uuid;

use crate::{
    features::accounts::{
        domain::account::{AccountProfile, ProfileUpdateCommand},
        error::AccountError,
        service::{
            account_service::AccountService,
            authentication::{MAX_USER_NAME_BYTES, password_within_auth_bound},
            password_work::PasswordBudget,
        },
    },
    util::string::validations::validate_username,
};

impl AccountService {
    pub async fn update_profile(
        &self,
        user_id: Uuid,
        current_password: &str,
        mut command: ProfileUpdateCommand,
    ) -> Result<AccountProfile, AccountError> {
        command.user_name = command.user_name.trim().to_owned();
        if !password_within_auth_bound(current_password) {
            return Err(AccountError::InvalidPassword);
        }
        if command.user_name.len() > MAX_USER_NAME_BYTES || !validate_username(&command.user_name) {
            return Err(AccountError::InvalidUserName);
        }

        let session_consistency_read = self.session_consistency.read().await;
        let candidate = self.repository.account_deletion_candidate(user_id).await?;
        let password_matches = self
            .verify_password(
                PasswordBudget::Confirmation,
                current_password,
                &candidate.password_hash,
            )
            .await?;
        if !password_matches {
            return Err(AccountError::WrongPassword);
        }
        drop(session_consistency_read);

        let session_consistency = self.session_consistency.write().await;
        let profile = self
            .repository
            .update_profile(user_id, &candidate.password_hash, &command, Utc::now())
            .await?;
        self.refresh_sessions_after_commit(user_id, "update_profile")
            .await;
        drop(session_consistency);
        Ok(profile)
    }
}
