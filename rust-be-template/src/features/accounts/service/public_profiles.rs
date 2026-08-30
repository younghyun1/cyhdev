//! Public account lookup use case.

use crate::{
    features::accounts::{
        domain::account::PublicAccount, error::AccountError,
        service::account_service::AccountService,
    },
    util::string::validations::validate_username,
};

impl AccountService {
    /// Returns the unique public account identified by an exact username.
    pub async fn public_account(&self, user_name: &str) -> Result<PublicAccount, AccountError> {
        let user_name = user_name.trim();
        if !validate_username(user_name) {
            return Err(AccountError::InvalidUserName);
        }

        match self
            .repository
            .public_account_by_user_name(user_name)
            .await?
        {
            Some(account) => Ok(account),
            None => Err(AccountError::AccountNotFound),
        }
    }
}
