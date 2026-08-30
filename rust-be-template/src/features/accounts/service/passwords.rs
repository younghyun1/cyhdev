//! Password-reset use cases.

use chrono::Utc;
use lettre::AsyncTransport;
use tracing::error;
use uuid::Uuid;
use zeroize::Zeroizing;

use crate::{
    DOMAIN_NAME,
    features::accounts::{
        domain::account::{PasswordResetReceipt, PasswordResetRequestReceipt},
        error::AccountError,
        service::{account_service::AccountService, authentication::validate_email},
    },
    util::{
        crypto::hash_pw::hash_pw, email::emails::PasswordResetEmail,
        string::validations::validate_password_form,
    },
};

const PASSWORD_RESET_TOKEN_VALID_DURATION: chrono::TimeDelta = chrono::Duration::minutes(30);

impl AccountService {
    pub async fn request_password_reset(
        &self,
        user_email: &str,
    ) -> Result<PasswordResetRequestReceipt, AccountError> {
        validate_email(user_email)?;
        let now = Utc::now();
        let token = Uuid::new_v4();
        let receipt = self
            .repository
            .issue_password_reset_token(
                user_email,
                token,
                now,
                now + PASSWORD_RESET_TOKEN_VALID_DURATION,
            )
            .await?;
        self.send_password_reset_email(receipt.user_email.clone(), receipt.token);
        Ok(receipt)
    }

    pub async fn reset_password(
        &self,
        token_value: Uuid,
        new_password: Zeroizing<String>,
    ) -> Result<PasswordResetReceipt, AccountError> {
        if !validate_password_form(&new_password) {
            return Err(AccountError::InvalidPassword);
        }
        let now = Utc::now();
        let token = match self.repository.password_reset_token(token_value).await? {
            Some(token) => token,
            None => return Err(AccountError::PasswordResetTokenNotFound),
        };
        if token.used_at.is_some() {
            return Err(AccountError::PasswordResetTokenAlreadyUsed);
        }
        if token.created_at > now {
            return Err(AccountError::PasswordResetTokenFabricated);
        }
        if token.expires_at < now {
            return Err(AccountError::PasswordResetTokenExpired);
        }

        let password_hash = hash_pw(new_password)
            .await
            .map_err(AccountError::PasswordHash)?;
        let _session_consistency = self.session_consistency.write().await;
        let receipt = match self
            .repository
            .consume_password_reset_token(&token, now, &password_hash)
            .await
        {
            Err(AccountError::TokenAlreadyConsumed) => {
                return Err(AccountError::PasswordResetTokenAlreadyUsed);
            }
            result => result?,
        };
        self.sessions.remove_for_user(receipt.user_id).await;
        Ok(receipt)
    }

    fn send_password_reset_email(&self, user_email: String, token: Uuid) {
        let email_client = self.email_client.clone();
        tokio::spawn(async move {
            let message = match PasswordResetEmail::new()
                .set_link(&format!(
                    "https://{DOMAIN_NAME}/reset-password?token={token}"
                ))
                .to_message(&user_email)
            {
                Ok(message) => message,
                Err(error) => {
                    error!(%error, "Could not build password reset email");
                    return;
                }
            };
            if let Err(error) = email_client.send(message).await {
                error!(%error, "Could not send password reset email");
            }
        });
    }
}
