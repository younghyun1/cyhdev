//! Account registration use case.

use chrono::Utc;
use lettre::AsyncTransport;
use std::sync::Arc;
use tracing::error;
use uuid::Uuid;

use crate::{
    features::accounts::{
        domain::account::{
            DuplicateRegistration, NewAccount, NewAccountRegistration, SignupCommand,
            SignupOutcome, SignupReceipt,
        },
        error::AccountError,
        service::{
            account_service::AccountService,
            authentication::{
                MAX_USER_NAME_BYTES, normalize_email, validate_auth_password, validate_email,
            },
            password_work::PasswordBudget,
            session_coordination::run_to_completion,
        },
    },
    util::{email::emails::ValidateEmailEmail, string::validations::validate_username},
};

const EMAIL_VERIFICATION_TOKEN_VALID_DURATION: chrono::TimeDelta = chrono::Duration::days(1);

impl AccountService {
    /// Registers an account, or settles a signup whose email already exists.
    ///
    /// Both paths hash the password first, so the work and the accepted response do not
    /// reveal whether the email was registered.
    pub async fn signup(
        self: &Arc<Self>,
        command: SignupCommand,
    ) -> Result<SignupOutcome, AccountError> {
        if command.user_name.len() > MAX_USER_NAME_BYTES || !validate_username(&command.user_name) {
            return Err(AccountError::InvalidUserName);
        }
        if !validate_auth_password(&command.password) {
            return Err(AccountError::InvalidPassword);
        }
        let user_email = normalize_email(&command.user_email);
        validate_email(&user_email)?;
        let now = Utc::now();
        let verification_token = Uuid::new_v4();
        let verify_by = now + EMAIL_VERIFICATION_TOKEN_VALID_DURATION;
        let password_hash = self
            .hash_password(PasswordBudget::Authentication, command.password)
            .await?;
        let registration = NewAccountRegistration {
            account: NewAccount {
                user_name: command.user_name,
                user_email,
                password_hash,
                country: command.country,
                language: command.language,
                subdivision: command.subdivision,
            },
            verification_token,
            verification_created_at: now,
            verification_expires_at: verify_by,
        };
        match self.repository.register_account(&registration).await {
            Ok(()) => {
                let receipt = self.issue_signup_receipt(registration);
                Ok(SignupOutcome::Registered(receipt))
            }
            Err(AccountError::DuplicateEmail(_)) => {
                self.replace_unverified_registration(registration).await
            }
            Err(error) => Err(error),
        }
    }

    async fn replace_unverified_registration(
        self: &Arc<Self>,
        registration: NewAccountRegistration,
    ) -> Result<SignupOutcome, AccountError> {
        let service = Arc::clone(self);
        run_to_completion(async move {
            // The replaced password must not survive in any session, so the commit and the
            // revocation happen together under the session-consistency write lock.
            let session_consistency = service.session_consistency.write().await;
            let settled = service
                .repository
                .replace_unverified_registration(&registration)
                .await?;
            match settled {
                DuplicateRegistration::ReplacedUnverified { user_id } => {
                    service.sessions.remove_for_user(user_id).await;
                    drop(session_consistency);
                    let receipt = service.issue_signup_receipt(registration);
                    Ok(SignupOutcome::ReplacedUnverified(receipt))
                }
                DuplicateRegistration::Unchanged => Ok(SignupOutcome::AlreadyVerified),
            }
        })
        .await
    }

    fn issue_signup_receipt(&self, registration: NewAccountRegistration) -> SignupReceipt {
        let NewAccountRegistration {
            account,
            verification_token,
            verification_expires_at,
            ..
        } = registration;
        self.send_verification_email(
            account.user_email.clone(),
            verification_token,
            verification_expires_at,
        );
        SignupReceipt {
            user_name: account.user_name,
            user_email: account.user_email,
            verify_by: verification_expires_at,
        }
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
        let public_app_origin = Arc::clone(&self.public_app_origin);
        tokio::spawn(async move {
            let _email_job = email_job;
            let message = match ValidateEmailEmail::new()
                .set_fields(verify_by, token, &public_app_origin)
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
