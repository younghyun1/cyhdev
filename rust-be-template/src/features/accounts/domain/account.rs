//! Persistence-independent account values passed between repository and service boundaries.

use chrono::{DateTime, Utc};
use uuid::Uuid;
use zeroize::Zeroizing;

/// Account data required to authenticate and seed a session.
pub struct LoginAccount {
    pub user_id: Uuid,
    pub user_name: String,
    pub password_hash: String,
    pub is_email_verified: bool,
    pub country: i32,
    pub language: i32,
}

/// Account data copied into an existing session after an account mutation.
#[derive(Debug, Clone)]
pub struct SessionAccount {
    pub user_name: String,
    pub is_email_verified: bool,
    pub country: i32,
    pub language: i32,
}

/// Public account data returned by the current-account use case.
#[derive(Debug, Clone)]
pub struct AccountProfile {
    pub user_id: Uuid,
    pub user_name: String,
    pub user_email: String,
    pub is_email_verified: bool,
    pub country: i32,
    pub language: i32,
    pub subdivision: Option<i32>,
}

/// Public account fields exposed by the username lookup use case.
#[derive(Debug, Clone)]
pub struct PublicAccount {
    pub user_id: Uuid,
    pub user_name: String,
    pub created_at: DateTime<Utc>,
    pub country: i32,
    pub profile_picture_url: Option<String>,
}

/// Most recent profile-picture metadata for an account.
#[derive(Debug, Clone)]
pub struct ProfilePicture {
    pub profile_picture_id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub image_type: i32,
    pub is_on_cloud: bool,
    pub link: Option<String>,
}

/// Metadata commit result used to remove superseded object-store files.
#[derive(Debug)]
pub struct ProfilePictureReplacement {
    pub profile_picture_id: Uuid,
    pub superseded_links: Vec<String>,
}

/// Aggregate returned by the current-account use case.
#[derive(Debug, Clone)]
pub struct CurrentAccount {
    pub profile: AccountProfile,
    pub profile_picture: Option<ProfilePicture>,
}

/// Registration input with a password that is zeroed on drop.
pub struct SignupCommand {
    pub user_name: String,
    pub user_email: String,
    pub password: Zeroizing<String>,
    pub country: i32,
    pub language: i32,
    pub subdivision: Option<i32>,
}

/// Registration data after password hashing.
pub struct NewAccount {
    pub user_name: String,
    pub user_email: String,
    pub password_hash: String,
    pub country: i32,
    pub language: i32,
    pub subdivision: Option<i32>,
}

/// Account and verification-token data committed atomically at signup.
pub struct NewAccountRegistration {
    pub account: NewAccount,
    pub verification_token: Uuid,
    pub verification_created_at: DateTime<Utc>,
    pub verification_expires_at: DateTime<Utc>,
}

/// Result of a successful signup.
#[derive(Debug, Clone)]
pub struct SignupReceipt {
    pub user_name: String,
    pub user_email: String,
    pub verify_by: DateTime<Utc>,
}

/// Password-reset token state used for service-level validation.
#[derive(Debug, Clone)]
pub struct PasswordResetToken {
    pub token_id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
}

/// Email-verification token state used for service-level validation.
#[derive(Debug, Clone)]
pub struct EmailVerificationToken {
    pub token_id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
}

/// Account fields returned after a password change.
#[derive(Debug, Clone)]
pub struct PasswordResetReceipt {
    pub user_id: Uuid,
    pub user_name: String,
    pub user_email: String,
    pub updated_at: DateTime<Utc>,
}

/// Result of issuing a password-reset token.
#[derive(Debug, Clone)]
pub struct PasswordResetRequestReceipt {
    pub user_email: String,
    pub token: Uuid,
    pub verify_by: DateTime<Utc>,
}

/// Result of consuming an email-verification token.
#[derive(Debug, Clone)]
pub struct EmailVerificationReceipt {
    pub user_id: Uuid,
    pub user_email: String,
    pub verified_at: DateTime<Utc>,
}

/// Result of successful credential authentication.
#[derive(Debug)]
pub struct LoginReceipt {
    pub user_id: Uuid,
    pub session_token: super::session::SessionToken,
}
