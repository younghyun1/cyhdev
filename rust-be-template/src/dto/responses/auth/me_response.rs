use chrono::{DateTime, Utc};
use serde_derive::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::features::accounts::domain::account::{
    AccountProfile, ProfilePicture as AccountProfilePicture,
};

#[derive(Serialize, ToSchema)]
pub struct UserInfo {
    pub user_id: Uuid,
    pub user_name: String,
    pub user_email: String,
    pub user_is_email_verified: bool,
    pub user_country: i32,
    pub user_language: i32,
    #[schema(required)]
    pub user_subdivision: Option<i32>,
}

impl From<AccountProfile> for UserInfo {
    fn from(profile: AccountProfile) -> Self {
        Self {
            user_id: profile.user_id,
            user_name: profile.user_name,
            user_email: profile.user_email,
            user_is_email_verified: profile.is_email_verified,
            user_country: profile.country,
            user_language: profile.language,
            user_subdivision: profile.subdivision,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct UserProfilePicture {
    pub user_profile_picture_id: Uuid,
    pub user_id: Uuid,
    pub user_profile_picture_created_at: DateTime<Utc>,
    pub user_profile_picture_updated_at: DateTime<Utc>,
    pub user_profile_picture_image_type: i32,
    pub user_profile_picture_is_on_cloud: bool,
    #[schema(required)]
    pub user_profile_picture_link: Option<String>,
}

impl From<AccountProfilePicture> for UserProfilePicture {
    fn from(profile_picture: AccountProfilePicture) -> Self {
        Self {
            user_profile_picture_id: profile_picture.profile_picture_id,
            user_id: profile_picture.user_id,
            user_profile_picture_created_at: profile_picture.created_at,
            user_profile_picture_updated_at: profile_picture.updated_at,
            user_profile_picture_image_type: profile_picture.image_type,
            user_profile_picture_is_on_cloud: profile_picture.is_on_cloud,
            user_profile_picture_link: profile_picture.link,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct MeResponse {
    #[schema(required)]
    pub user_info: Option<UserInfo>,
    #[schema(required)]
    pub user_profile_picture: Option<UserProfilePicture>,
    pub build_time: &'static str,
    pub axum_version: String,
    pub rust_version: &'static str,
}
