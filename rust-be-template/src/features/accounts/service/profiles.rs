//! Current-account and profile-picture use cases.

use uuid::Uuid;

use crate::features::accounts::{
    domain::account::{CurrentAccount, ProfilePictureReplacement},
    error::AccountError,
    service::account_service::AccountService,
};

impl AccountService {
    pub async fn current_account(
        &self,
        user_id: Uuid,
    ) -> Result<Option<CurrentAccount>, AccountError> {
        self.repository.current_account(user_id).await
    }

    pub async fn replace_profile_picture_metadata(
        &self,
        user_id: Uuid,
        image_type: i32,
        link: &str,
    ) -> Result<ProfilePictureReplacement, AccountError> {
        self.repository
            .replace_profile_picture(user_id, image_type, true, Some(link))
            .await
    }
}
