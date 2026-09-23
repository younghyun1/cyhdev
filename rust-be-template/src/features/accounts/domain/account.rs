//! Persistence-independent account values passed between repository and service boundaries.

use chrono::{DateTime, Utc};
use uuid::Uuid;
use zeroize::Zeroizing;

use crate::util::media::cleanup::DurableMediaCleanup;

/// Public label for retained content whose author deleted their account.
pub const DELETED_USER_DISPLAY_NAME: &str = "Deleted user";

/// Account data required to authenticate and seed a session.
pub struct LoginAccount {
    pub user_id: Uuid,
    pub user_name: String,
    pub password_hash: String,
    pub is_email_verified: bool,
    pub country: i32,
    pub language: i32,
}

impl LoginAccount {
    pub fn session_principal(&self) -> SessionPrincipal {
        SessionPrincipal {
            user_id: self.user_id,
            user_name: self.user_name.clone(),
            is_email_verified: self.is_email_verified,
            country: self.country,
            language: self.language,
        }
    }
}

/// Account authority used to seed a process-local session.
#[derive(Debug, Clone)]
pub struct SessionPrincipal {
    pub user_id: Uuid,
    pub user_name: String,
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

/// One retained profile picture; at most one row per account is active.
#[derive(Debug, Clone)]
pub struct ProfilePicture {
    pub profile_picture_id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub image_type: i32,
    pub is_on_cloud: bool,
    pub is_active: bool,
    pub link: Option<String>,
}

/// Metadata commit result used to remove superseded object-store files.
#[derive(Debug)]
pub struct ProfilePictureReplacement {
    pub profile_picture_id: Uuid,
    /// Resolved durable cleanup rows which may be attempted immediately.
    pub cleanup_objects: Vec<DurableMediaCleanup>,
    /// Legacy URLs retained for explicit administrative resolution.
    pub unresolved_cleanup_count: usize,
}

/// Result of deleting one owned profile-picture history entry.
#[derive(Debug)]
pub struct ProfilePictureDeletion {
    pub deleted_profile_picture_id: Uuid,
    pub active_profile_picture_id: Option<Uuid>,
    pub cleanup_objects: Vec<DurableMediaCleanup>,
    pub unresolved_cleanup_count: usize,
}

/// Aggregate returned by the current-account use case.
#[derive(Debug, Clone)]
pub struct CurrentAccount {
    pub profile: AccountProfile,
    pub profile_picture: Option<ProfilePicture>,
}

/// Editable full-profile fields; email remains immutable through this use case.
pub struct ProfileUpdateCommand {
    pub user_name: String,
    pub country: i32,
    pub language: i32,
    pub subdivision: Option<i32>,
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

/// How a signup that collided with an existing email was settled.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum DuplicateRegistration {
    /// The unverified account now holds the new submission's credentials and name.
    ReplacedUnverified { user_id: Uuid },
    /// The address belongs to a verified account, which is left untouched.
    Unchanged,
}

/// Public result of a signup; every variant returns the same accepted response.
#[derive(Debug, Clone)]
pub enum SignupOutcome {
    Registered(SignupReceipt),
    /// A second signup for an unverified address replaced its password hash and user name and
    /// invalidated the earlier verification link, so whoever registered the address first
    /// cannot keep a password on the account its owner later verifies.
    ReplacedUnverified(SignupReceipt),
    /// The address is already verified; nothing changed and no email was sent.
    AlreadyVerified,
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
