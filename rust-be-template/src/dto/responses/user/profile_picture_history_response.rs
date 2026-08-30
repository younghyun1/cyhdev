use chrono::{DateTime, Utc};
use serde_derive::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::features::accounts::domain::account::ProfilePicture;

#[derive(Debug, Serialize, ToSchema)]
pub struct ProfilePictureHistoryItem {
    pub profile_picture_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub image_type: i32,
    pub is_active: bool,
    pub object_url: String,
}

impl From<ProfilePicture> for ProfilePictureHistoryItem {
    fn from(profile_picture: ProfilePicture) -> Self {
        let object_url = profile_picture.link.unwrap_or_default();
        Self {
            profile_picture_id: profile_picture.profile_picture_id,
            created_at: profile_picture.created_at,
            updated_at: profile_picture.updated_at,
            image_type: profile_picture.image_type,
            is_active: profile_picture.is_active,
            object_url,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProfilePictureHistoryResponse {
    pub profile_pictures: Vec<ProfilePictureHistoryItem>,
    pub maximum_profile_pictures: usize,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SelectProfilePictureResponse {
    pub profile_picture: ProfilePictureHistoryItem,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DeleteProfilePictureResponse {
    pub deleted_profile_picture_id: Uuid,
    #[schema(required)]
    pub active_profile_picture_id: Option<Uuid>,
    pub cleanup_deleted_count: usize,
    pub cleanup_failure_count: usize,
    pub cleanup_remaining_count: usize,
}
