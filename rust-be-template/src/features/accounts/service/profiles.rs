//! Current-account and profile-picture use cases.

use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::features::accounts::{
    domain::account::{
        CurrentAccount, ProfilePicture, ProfilePictureDeletion, ProfilePictureReplacement,
    },
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

    pub async fn profile_picture_history(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<ProfilePicture>, AccountError> {
        self.repository.profile_picture_history(user_id).await
    }

    pub async fn select_profile_picture(
        &self,
        user_id: Uuid,
        profile_picture_id: Uuid,
    ) -> Result<Option<ProfilePicture>, AccountError> {
        self.repository
            .select_profile_picture(user_id, profile_picture_id)
            .await
    }

    pub async fn delete_profile_picture(
        &self,
        user_id: Uuid,
        profile_picture_id: Uuid,
    ) -> Result<Option<ProfilePictureDeletion>, AccountError> {
        self.repository
            .delete_profile_picture(user_id, profile_picture_id)
            .await
    }

    pub async fn complete_media_object_cleanup(
        &self,
        cleanup_id: Uuid,
    ) -> Result<bool, AccountError> {
        self.repository
            .complete_media_object_cleanup(cleanup_id)
            .await
    }

    pub async fn record_media_object_cleanup_failure(
        &self,
        cleanup_id: Uuid,
        expected_attempt_count: i32,
        attempted_at: DateTime<Utc>,
        error: &str,
    ) -> Result<bool, AccountError> {
        self.repository
            .record_media_object_cleanup_failure(
                cleanup_id,
                expected_attempt_count,
                attempted_at,
                error,
            )
            .await
    }
}
