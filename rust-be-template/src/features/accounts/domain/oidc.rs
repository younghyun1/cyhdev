//! Persistence-independent OpenID Connect account values.

use uuid::Uuid;
use zeroize::Zeroizing;

use super::{account::SessionPrincipal, session::SessionToken};

pub const MAX_OIDC_ISSUER_BYTES: usize = 1_024;
pub const MAX_OIDC_SUBJECT_BYTES: usize = 255;
pub const MAX_OIDC_PROVIDER_EMAIL_BYTES: usize = 254;
/// Host-only cookie that binds a pending authorization to the browser that started it.
pub const OIDC_BINDING_COOKIE_NAME: &str = "__Host-cyhdev-oidc-binding";
/// Matches the pending-flow lifetime; the callback clears the cookie either way.
pub const OIDC_BINDING_COOKIE_SECONDS: i64 = 10 * 60;

/// Verified identity claims retained only after signature and nonce validation.
#[derive(Clone)]
pub struct OidcIdentityClaims {
    pub issuer: String,
    pub subject: String,
    pub provider_email: String,
}

/// Purpose attached to a one-time authorization state entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OidcFlowMode {
    Login,
    Link { expected_user_id: Uuid },
}

/// Active local account resolved exclusively through issuer and subject.
pub struct OidcAccount {
    pub user_id: Uuid,
    pub user_name: String,
    pub is_email_verified: bool,
    pub country: i32,
    pub language: i32,
}

impl From<OidcAccount> for SessionPrincipal {
    fn from(account: OidcAccount) -> Self {
        Self {
            user_id: account.user_id,
            user_name: account.user_name,
            is_email_verified: account.is_email_verified,
            country: account.country,
            language: account.language,
        }
    }
}

/// Committed link with what the owner notice needs.
pub struct OidcLinkReceipt {
    pub principal: SessionPrincipal,
    pub owner_email: String,
    /// False when the same issuer and subject were already linked and only refreshed.
    pub newly_linked: bool,
}

/// Credential snapshot used to confirm an unlink transaction.
pub struct OidcUnlinkCandidate {
    pub password_hash: Zeroizing<String>,
}

/// A successful OIDC account operation with a rotated browser session.
pub struct OidcSessionReceipt {
    pub user_id: Uuid,
    pub session_token: SessionToken,
}
