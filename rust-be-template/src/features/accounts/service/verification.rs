//! Email-verification use case.

use chrono::Utc;
use uuid::Uuid;

use crate::features::accounts::{
    domain::account::EmailVerificationReceipt, error::AccountError,
    service::account_service::AccountService,
};

impl AccountService {
    pub async fn verify_email(
        &self,
        token_value: Uuid,
    ) -> Result<EmailVerificationReceipt, AccountError> {
        let now = Utc::now();
        let token = match self
            .repository
            .email_verification_token(token_value)
            .await?
        {
            Some(token) => token,
            None => return Err(AccountError::EmailVerificationTokenNotFound),
        };
        if token.used_at.is_some() {
            return Err(AccountError::EmailVerificationTokenAlreadyUsed);
        }
        if token.created_at > now {
            return Err(AccountError::EmailVerificationTokenFabricated);
        }
        if token.expires_at < now {
            return Err(AccountError::EmailVerificationTokenExpired);
        }

        let _session_consistency = self.session_consistency.write().await;
        let receipt = match self
            .repository
            .consume_email_verification_token(&token, now)
            .await
        {
            Err(AccountError::TokenAlreadyConsumed) => {
                return Err(AccountError::EmailVerificationTokenAlreadyUsed);
            }
            result => result?,
        };
        self.refresh_sessions_after_commit(receipt.user_id, "verify_email")
            .await;
        Ok(receipt)
    }
}
