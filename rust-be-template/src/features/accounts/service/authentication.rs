//! Credential and session use cases.

use crate::{
    features::accounts::{
        domain::{account::LoginReceipt, role::RoleType},
        error::AccountError,
        service::account_service::AccountService,
    },
    util::{crypto::verify_pw::verify_pw, string::validations::validate_password_form},
};

impl AccountService {
    pub async fn email_exists(&self, email: &str) -> Result<bool, AccountError> {
        validate_email(email)?;
        self.repository.email_exists(email).await
    }

    pub async fn login(
        &self,
        email: &str,
        password: &str,
        previous_session_token: Option<&str>,
    ) -> Result<LoginReceipt, AccountError> {
        validate_email(email)?;
        if !validate_password_form(password) {
            return Err(AccountError::InvalidPassword);
        }

        // A session-affecting mutation takes the write side through its commit and
        // refresh/revocation. Holding the read side through session creation prevents
        // an in-flight login from recreating a session with stale account state.
        let _session_consistency = self.session_consistency.read().await;
        let account = match self.repository.login_account_by_email(email).await? {
            Some(account) => account,
            None => return Err(AccountError::AccountNotFound),
        };
        let password_matches = verify_pw(password, &account.password_hash)
            .await
            .map_err(AccountError::PasswordVerification)?;
        if !password_matches {
            return Err(AccountError::WrongPassword);
        }

        let role_type = self
            .repository
            .role_for_user_or_insert_default(account.user_id, RoleType::User)
            .await?;
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

pub(super) fn validate_email(email: &str) -> Result<(), AccountError> {
    if email_address::EmailAddress::is_valid(email) {
        Ok(())
    } else {
        Err(AccountError::InvalidEmail)
    }
}
