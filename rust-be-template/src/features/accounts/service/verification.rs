//! Email-verification use case.

use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::features::accounts::{
    domain::account::EmailVerificationReceipt,
    error::AccountError,
    service::{account_service::AccountService, session_coordination::run_to_completion},
};

impl AccountService {
    pub async fn verify_email(
        self: &Arc<Self>,
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

        let service = Arc::clone(self);
        run_to_completion(async move {
            let _session_consistency = service.session_consistency.write().await;
            let receipt = match service
                .repository
                .consume_email_verification_token(&token, now)
                .await
            {
                Err(AccountError::TokenAlreadyConsumed) => {
                    return Err(AccountError::EmailVerificationTokenAlreadyUsed);
                }
                result => result?,
            };
            // A session opened before verification could belong to whoever registered the
            // address first, so verification ends every session instead of upgrading it.
            let revoked = service.sessions.remove_for_user(receipt.user_id).await;
            tracing::trace!(user_id = %receipt.user_id, revoked, "Revoked sessions after email verification");
            Ok(receipt)
        })
        .await
    }
}
