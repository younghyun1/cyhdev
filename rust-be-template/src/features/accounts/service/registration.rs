//! Account registration use case.

use chrono::Utc;
use lettre::AsyncTransport;
use tracing::error;
use uuid::Uuid;

use crate::{
    features::accounts::{
        domain::account::{NewAccount, NewAccountRegistration, SignupCommand, SignupReceipt},
        error::AccountError,
        service::{
            account_service::AccountService,
            authentication::{
                MAX_USER_NAME_BYTES, validate_auth_password, validate_email,
            },
        },
    },
    util::{
        crypto::hash_pw::hash_pw,
        email::emails::ValidateEmailEmail,
        string::validations::validate_username,
    },
};

const EMAIL_VERIFICATION_TOKEN_VALID_DURATION: chrono::TimeDelta = chrono::Duration::days(1);

impl AccountService {
    pub async fn signup(&self, command: SignupCommand) -> Result<SignupReceipt, AccountError> {
        if command.user_name.len() > MAX_USER_NAME_BYTES
            || !validate_username(&command.user_name)
        {
            return Err(AccountError::InvalidUserName);
        }
        if !validate_auth_password(&command.password) {
            return Err(AccountError::InvalidPassword);
        }
        validate_email(&command.user_email)?;
        let now = Utc::now();
        let verification_token = Uuid::new_v4();
        let verify_by = now + EMAIL_VERIFICATION_TOKEN_VALID_DURATION;
        let password_job = self.try_password_job()?;
        let password_hash = hash_pw(command.password)
            .await
            .map_err(AccountError::PasswordHash)?;
        drop(password_job);
        let registration = NewAccountRegistration {
            account: NewAccount {
                user_name: command.user_name.clone(),
                user_email: command.user_email.clone(),
                password_hash,
                country: command.country,
                language: command.language,
                subdivision: command.subdivision,
            },
            verification_token,
            verification_created_at: now,
            verification_expires_at: verify_by,
        };
        self.repository.register_account(&registration).await?;

        self.send_verification_email(command.user_email.clone(), verification_token, verify_by);
        Ok(SignupReceipt {
            user_name: command.user_name,
            user_email: command.user_email,
            verify_by,
        })
    }

    pub async fn resend_verification_for_duplicate_email(
        &self,
        user_email: &str,
    ) -> Result<(), AccountError> {
        validate_email(user_email)?;
        let now = Utc::now();
        let issue = self
            .repository
            .replace_email_verification_token_if_unverified(
                user_email,
                Uuid::new_v4(),
                now,
                now + EMAIL_VERIFICATION_TOKEN_VALID_DURATION,
            )
            .await?;
        if let Some(issue) = issue {
            self.send_verification_email(issue.user_email, issue.token, issue.verify_by);
        }
        Ok(())
    }

    fn send_verification_email(
        &self,
        user_email: String,
        token: Uuid,
        verify_by: chrono::DateTime<Utc>,
    ) {
        let email_job = match self.email_jobs.clone().try_acquire_owned() {
            Ok(email_job) => email_job,
            Err(_) => {
                tracing::warn!(
                    event = "auth_email_work_rejected",
                    email_kind = "verification",
                    max_jobs = crate::features::accounts::service::account_service::MAX_EMAIL_JOBS,
                    "Authentication email work rejected"
                );
                return;
            }
        };
        let email_client = self.email_client.clone();
        tokio::spawn(async move {
            let _email_job = email_job;
            let message = match ValidateEmailEmail::new()
                .set_fields(verify_by, token)
                .to_message(&user_email)
            {
                Ok(message) => message,
                Err(error) => {
                    error!(%error, "Could not build validation email");
                    return;
                }
            };
            if let Err(error) = email_client.send(message).await {
                error!(%error, "Could not send validation email");
            }
        });
    }
}
