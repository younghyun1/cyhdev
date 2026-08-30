//! Public identity state for retained authored content.

use uuid::Uuid;

use super::account::DELETED_USER_DISPLAY_NAME;

#[derive(Clone)]
pub struct PublicAuthor {
    public_user_id: Uuid,
    user_name: String,
    country_code: Option<i32>,
    profile_picture_url: String,
    deleted: bool,
}

impl PublicAuthor {
    pub(crate) fn active(
        user_id: Uuid,
        user_name: String,
        country_code: i32,
        profile_picture_url: Option<String>,
    ) -> Self {
        Self {
            public_user_id: user_id,
            user_name,
            country_code: Some(country_code),
            profile_picture_url: profile_picture_url.unwrap_or_default(),
            deleted: false,
        }
    }

    pub(crate) fn deleted() -> Self {
        Self {
            public_user_id: Uuid::nil(),
            user_name: DELETED_USER_DISPLAY_NAME.to_owned(),
            country_code: None,
            profile_picture_url: String::new(),
            deleted: true,
        }
    }

    pub fn public_user_id(&self) -> Uuid {
        self.public_user_id
    }

    pub fn user_name(&self) -> &str {
        &self.user_name
    }

    pub fn country_code(&self) -> Option<i32> {
        self.country_code
    }

    pub fn profile_picture_url(&self) -> &str {
        &self.profile_picture_url
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted
    }
}
