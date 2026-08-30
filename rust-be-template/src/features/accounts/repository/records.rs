//! Diesel records kept private to the account repository boundary.

use chrono::{DateTime, Utc};
use diesel::{AsChangeset, Insertable, Queryable, Selectable};
use uuid::Uuid;

use crate::{
    features::accounts::domain::account::{
        AccountProfile, EmailVerificationToken, LoginAccount, PasswordResetReceipt,
        PasswordResetToken, ProfilePicture, PublicAccount, SessionAccount,
    },
    schema::{
        email_verification_tokens, password_reset_tokens, user_profile_pictures, user_roles, users,
    },
};

#[derive(Queryable, Selectable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(super) struct AccountRecord {
    pub(super) user_id: Uuid,
    pub(super) user_name: String,
    pub(super) user_email: String,
    pub(super) user_password_hash: String,
    pub(super) user_updated_at: DateTime<Utc>,
    pub(super) user_is_email_verified: bool,
    pub(super) user_country: i32,
    pub(super) user_language: i32,
}

impl AccountRecord {
    pub(super) fn into_login_account(self) -> LoginAccount {
        LoginAccount {
            user_id: self.user_id,
            user_name: self.user_name,
            password_hash: self.user_password_hash,
            is_email_verified: self.user_is_email_verified,
            country: self.user_country,
            language: self.user_language,
        }
    }

    pub(super) fn into_password_reset_receipt(self) -> PasswordResetReceipt {
        PasswordResetReceipt {
            user_id: self.user_id,
            user_name: self.user_name,
            user_email: self.user_email,
            updated_at: self.user_updated_at,
        }
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(super) struct AccountProfileRecord {
    user_id: Uuid,
    user_name: String,
    user_email: String,
    user_is_email_verified: bool,
    user_country: i32,
    user_language: i32,
    user_subdivision: Option<i32>,
}

impl From<AccountProfileRecord> for AccountProfile {
    fn from(record: AccountProfileRecord) -> Self {
        Self {
            user_id: record.user_id,
            user_name: record.user_name,
            user_email: record.user_email,
            is_email_verified: record.user_is_email_verified,
            country: record.user_country,
            language: record.user_language,
            subdivision: record.user_subdivision,
        }
    }
}

impl From<AccountProfileRecord> for SessionAccount {
    fn from(record: AccountProfileRecord) -> Self {
        Self {
            user_name: record.user_name,
            is_email_verified: record.user_is_email_verified,
            country: record.user_country,
            language: record.user_language,
        }
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(super) struct PublicAccountRecord {
    user_id: Uuid,
    user_name: String,
    user_created_at: DateTime<Utc>,
    user_country: i32,
}

impl PublicAccountRecord {
    pub(super) fn user_id(&self) -> Uuid {
        self.user_id
    }

    pub(super) fn into_public_account(self, profile_picture_url: Option<String>) -> PublicAccount {
        PublicAccount {
            user_id: self.user_id,
            user_name: self.user_name,
            created_at: self.user_created_at,
            country: self.user_country,
            profile_picture_url,
        }
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = user_profile_pictures)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(super) struct ProfilePictureRecord {
    user_profile_picture_id: Uuid,
    user_id: Uuid,
    user_profile_picture_created_at: DateTime<Utc>,
    user_profile_picture_updated_at: DateTime<Utc>,
    user_profile_picture_image_type: i32,
    user_profile_picture_is_on_cloud: bool,
    user_profile_picture_is_active: bool,
    user_profile_picture_link: Option<String>,
}

impl From<ProfilePictureRecord> for ProfilePicture {
    fn from(record: ProfilePictureRecord) -> Self {
        Self {
            profile_picture_id: record.user_profile_picture_id,
            user_id: record.user_id,
            created_at: record.user_profile_picture_created_at,
            updated_at: record.user_profile_picture_updated_at,
            image_type: record.user_profile_picture_image_type,
            is_on_cloud: record.user_profile_picture_is_on_cloud,
            is_active: record.user_profile_picture_is_active,
            link: record.user_profile_picture_link,
        }
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = email_verification_tokens)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(super) struct EmailVerificationTokenRecord {
    email_verification_token_id: Uuid,
    user_id: Uuid,
    email_verification_token_expires_at: DateTime<Utc>,
    email_verification_token_created_at: DateTime<Utc>,
    email_verification_token_used_at: Option<DateTime<Utc>>,
}

impl From<EmailVerificationTokenRecord> for EmailVerificationToken {
    fn from(record: EmailVerificationTokenRecord) -> Self {
        Self {
            token_id: record.email_verification_token_id,
            user_id: record.user_id,
            created_at: record.email_verification_token_created_at,
            expires_at: record.email_verification_token_expires_at,
            used_at: record.email_verification_token_used_at,
        }
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = password_reset_tokens)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(super) struct PasswordResetTokenRecord {
    password_reset_token_id: Uuid,
    user_id: Uuid,
    password_reset_token_expires_at: DateTime<Utc>,
    password_reset_token_created_at: DateTime<Utc>,
    password_reset_token_used_at: Option<DateTime<Utc>>,
}

impl From<PasswordResetTokenRecord> for PasswordResetToken {
    fn from(record: PasswordResetTokenRecord) -> Self {
        Self {
            token_id: record.password_reset_token_id,
            user_id: record.user_id,
            created_at: record.password_reset_token_created_at,
            expires_at: record.password_reset_token_expires_at,
            used_at: record.password_reset_token_used_at,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub(super) struct NewAccountRecord<'a> {
    pub(super) user_name: &'a str,
    pub(super) user_email: &'a str,
    pub(super) user_password_hash: &'a str,
    pub(super) user_country: i32,
    pub(super) user_language: i32,
    pub(super) user_subdivision: Option<i32>,
}

#[derive(Insertable)]
#[diesel(table_name = user_roles)]
pub(super) struct NewUserRoleRecord {
    pub(super) user_id: Uuid,
    pub(super) role_id: Uuid,
}

#[derive(Insertable)]
#[diesel(table_name = email_verification_tokens)]
pub(super) struct NewEmailVerificationTokenRecord {
    pub(super) user_id: Uuid,
    pub(super) email_verification_token: Uuid,
    pub(super) email_verification_token_expires_at: DateTime<Utc>,
    pub(super) email_verification_token_created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = password_reset_tokens)]
pub(super) struct NewPasswordResetTokenRecord {
    pub(super) user_id: Uuid,
    pub(super) password_reset_token: Uuid,
    pub(super) password_reset_token_expires_at: DateTime<Utc>,
    pub(super) password_reset_token_created_at: DateTime<Utc>,
}

#[derive(AsChangeset)]
#[diesel(table_name = users)]
pub(super) struct UpdatePasswordRecord<'a> {
    pub(super) user_password_hash: &'a str,
    pub(super) user_updated_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = user_profile_pictures)]
pub(super) struct NewProfilePictureRecord<'a> {
    pub(super) user_id: Uuid,
    pub(super) user_profile_picture_image_type: i32,
    pub(super) user_profile_picture_is_on_cloud: bool,
    pub(super) user_profile_picture_is_active: bool,
    pub(super) user_profile_picture_link: Option<&'a str>,
}
