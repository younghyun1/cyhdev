//! Password-reset use cases.

use chrono::Utc;
use lettre::AsyncTransport;
use tracing::error;
use uuid::Uuid;
use zeroize::Zeroizing;

use crate::{
    DOMAIN_NAME,
    features::accounts::{
        domain::account::PasswordResetReceipt,
        error::AccountError,
        service::{
            account_service::AccountService,
            authentication::{validate_auth_password, validate_email},
        },
    },
    util::{
        crypto::{hash_pw::hash_pw, verify_pw::verify_pw},
        email::emails::PasswordResetEmail,
    },
};

const PASSWORD_RESET_TOKEN_VALID_DURATION: chrono::TimeDelta = chrono::Duration::minutes(30);
const DUMMY_RESET_PASSWORD: &str = "ResetTimingOnly5728";

impl AccountService {
    pub async fn request_password_reset(
        &self,
        user_email: &str,
    ) -> Result<(), AccountError> {
        validate_email(user_email)?;
        let password_job = self.try_password_job()?;
        let _password_matches = verify_pw(DUMMY_RESET_PASSWORD, &self.dummy_password_hash)
            .await
            .map_err(AccountError::PasswordVerification)?;
        drop(password_job);

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
        if let Some(receipt) = receipt {
            self.send_password_reset_email(receipt.user_email, receipt.token);
        }
        Ok(())
    }

    pub async fn reset_password(
        &self,
        token_value: Uuid,
        new_password: Zeroizing<String>,
    ) -> Result<PasswordResetReceipt, AccountError> {
        if !validate_auth_password(&new_password) {
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

        let password_job = self.try_password_job()?;
        let password_hash = hash_pw(new_password)
            .await
            .map_err(AccountError::PasswordHash)?;
        drop(password_job);
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
        let email_job = match self.email_jobs.clone().try_acquire_owned() {
            Ok(email_job) => email_job,
            Err(_) => {
                tracing::warn!(
                    event = "auth_email_work_rejected",
                    email_kind = "password_reset",
                    max_jobs = crate::features::accounts::service::account_service::MAX_EMAIL_JOBS,
                    "Authentication email work rejected"
                );
                return;
            }
        };
        let email_client = self.email_client.clone();
        tokio::spawn(async move {
            let _email_job = email_job;
            let message = match PasswordResetEmail::new()
                .set_link(&format!(
                    "https://{DOMAIN_NAME}/reset-password#token={token}"
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
