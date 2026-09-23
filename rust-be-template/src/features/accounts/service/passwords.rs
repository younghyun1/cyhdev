//! Password-reset use cases.

use chrono::Utc;
use lettre::AsyncTransport;
use std::sync::Arc;
use tracing::error;
use zeroize::Zeroizing;

use crate::{
    features::accounts::{
        domain::{
            account::PasswordResetReceipt,
            capability_token::{CapabilityDigest, CapabilityToken},
        },
        error::AccountError,
        service::{
            account_service::AccountService,
            authentication::{normalize_email, validate_auth_password, validate_email},
            password_work::PasswordBudget,
            session_coordination::run_to_completion,
        },
    },
    util::email::emails::PasswordResetEmail,
};

const PASSWORD_RESET_TOKEN_VALID_DURATION: chrono::TimeDelta = chrono::Duration::minutes(30);
const DUMMY_RESET_PASSWORD: &str = "ResetTimingOnly5728";

impl AccountService {
    pub async fn request_password_reset(&self, user_email: &str) -> Result<(), AccountError> {
        let user_email = normalize_email(user_email);
        let user_email = user_email.as_str();
        validate_email(user_email)?;
        let _password_matches = self
            .verify_password(
                PasswordBudget::Authentication,
                DUMMY_RESET_PASSWORD,
                &self.dummy_password_hash,
            )
            .await?;

        let now = Utc::now();
        let (token, digest) =
            CapabilityToken::generate().map_err(AccountError::CapabilityEntropy)?;
        let receipt = self
            .repository
            .issue_password_reset_token(
                user_email,
                &digest,
                now,
                now + PASSWORD_RESET_TOKEN_VALID_DURATION,
            )
            .await?;
        if let Some(receipt) = receipt {
            self.send_password_reset_email(receipt.user_email, &token);
        }
        Ok(())
    }

    pub async fn reset_password(
        self: &Arc<Self>,
        token_value: &str,
        new_password: Zeroizing<String>,
    ) -> Result<PasswordResetReceipt, AccountError> {
        if !validate_auth_password(&new_password) {
            return Err(AccountError::InvalidPassword);
        }
        let digest = CapabilityDigest::from_submitted(token_value)
            .ok_or(AccountError::PasswordResetTokenNotFound)?;
        let now = Utc::now();
        let token = match self.repository.password_reset_token(&digest).await? {
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

        let password_hash = self
            .hash_password(PasswordBudget::Authentication, new_password)
            .await?;
        let service = Arc::clone(self);
        run_to_completion(async move {
            let _session_consistency = service.session_consistency.write().await;
            let receipt = match service
                .repository
                .consume_password_reset_token(&token, now, &password_hash)
                .await
            {
                Err(AccountError::TokenAlreadyConsumed) => {
                    return Err(AccountError::PasswordResetTokenAlreadyUsed);
                }
                result => result?,
            };
            service.sessions.remove_for_user(receipt.user_id).await;
            Ok(receipt)
        })
        .await
    }

    fn send_password_reset_email(&self, user_email: String, token: &CapabilityToken) {
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
        let link = Zeroizing::new(format!(
            "{}/reset-password#token={}",
            self.public_app_origin,
            token.expose()
        ));
        tokio::spawn(async move {
            let _email_job = email_job;
            let message = match PasswordResetEmail::new()
                .set_link(&link)
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
