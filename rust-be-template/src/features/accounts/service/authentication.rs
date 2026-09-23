//! Credential and session use cases.

use crate::{
    features::accounts::{
        domain::{
            account::{LoginCandidate, LoginReceipt},
            role::RoleType,
        },
        error::AccountError,
        service::{account_service::AccountService, password_work::PasswordBudget},
    },
    util::string::validations::validate_password_form,
};

pub const MAX_EMAIL_BYTES: usize = 254;
pub const MAX_PASSWORD_BYTES: usize = 128;
pub const MAX_USER_NAME_BYTES: usize = 80;

impl AccountService {
    pub async fn email_exists(&self, email: &str) -> Result<bool, AccountError> {
        let email = normalize_email(email);
        validate_email(&email)?;
        self.repository.email_exists(&email).await
    }

    pub async fn login(
        &self,
        email: &str,
        password: &str,
        previous_session_token: Option<&str>,
    ) -> Result<LoginReceipt, AccountError> {
        let email = normalize_email(email);
        let email = email.as_str();
        validate_email(email)?;
        if !validate_auth_password(password) {
            return Err(AccountError::InvalidPassword);
        }

        // A session-affecting mutation takes the write side through its commit and
        // refresh/revocation. Holding the read side through session creation prevents
        // an in-flight login from recreating a session with stale account state.
        let _session_consistency = self.session_consistency.read().await;
        let candidate = self.repository.login_account_by_email(email).await?;
        let expected_hash = match &candidate {
            Some(candidate) => candidate.account.password_hash.as_str(),
            None => self.dummy_password_hash.as_ref(),
        };
        let password_matches = self
            .verify_password(PasswordBudget::Authentication, password, expected_hash)
            .await?;
        let LoginCandidate { account, role } = match candidate {
            Some(candidate) if password_matches => candidate,
            Some(_) | None => return Err(AccountError::InvalidCredentials),
        };
        // Only a correct password learns that verification is pending. Until the email owner
        // verifies, the account may belong to whoever registered the address first, so it
        // never receives a session.
        if !account.is_email_verified {
            return Err(AccountError::EmailNotVerified);
        }

        let role_type = match role {
            Some(role_type) => role_type,
            None => {
                self.repository
                    .role_for_user_or_insert_default(account.user_id, RoleType::User)
                    .await?
            }
        };
        let session_token = self
            .sessions
            .create(&account, role_type, previous_session_token, None)
            .await?;
        Ok(LoginReceipt {
            user_id: account.user_id,
            session_token,
        })
    }

    pub async fn logout(&self, session_token: &str) -> bool {
        self.sessions.remove(session_token).await
    }
}

/// Canonical stored and compared form of an email address.
///
/// Emails are stored lowercase and compared with `lower()` in SQL, so the address a person
/// types in any letter case reaches the same account. Trimming matches the abuse limiter.
pub(super) fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

pub(super) fn validate_email(email: &str) -> Result<(), AccountError> {
    if email.len() <= MAX_EMAIL_BYTES && email_address::EmailAddress::is_valid(email) {
        Ok(())
    } else {
        Err(AccountError::InvalidEmail)
    }
}

pub(super) fn validate_auth_password(password: &str) -> bool {
    password_within_auth_bound(password) && validate_password_form(password)
}

pub(super) fn password_within_auth_bound(password: &str) -> bool {
    password.len() <= MAX_PASSWORD_BYTES
}
