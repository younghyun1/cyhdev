//! Process-local session credentials and authenticated account state.

use std::{fmt, sync::Arc};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use jiff::SignedDuration;
use sha2::{Digest, Sha256};
use uuid::Uuid;
use zeroize::Zeroizing;

use super::role::RoleType;

pub const DEFAULT_SESSION_DURATION: SignedDuration = SignedDuration::from_hours(1);
pub const SESSION_COOKIE_NAME: &str = "__Host-cyhdev-session";
pub const SESSION_SECRET_BYTES: usize = 32;
pub const SESSION_TOKEN_LENGTH: usize = 43;
const SESSION_DECODE_BUFFER_BYTES: usize = 33;

/// An opaque bearer credential returned only to the browser that owns the session.
pub struct SessionToken(Zeroizing<String>);

impl SessionToken {
    pub(crate) fn from_secret(secret: &[u8; SESSION_SECRET_BYTES]) -> Self {
        Self(Zeroizing::new(URL_SAFE_NO_PAD.encode(secret)))
    }

    /// Exposes the token only for transport in the secure session cookie.
    pub fn expose(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Debug for SessionToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SessionToken([REDACTED])")
    }
}

/// Fixed-size lookup key derived from a session secret.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub(crate) struct SessionKey([u8; SESSION_SECRET_BYTES]);

impl SessionKey {
    pub(crate) fn from_secret(secret: &[u8]) -> Self {
        let digest = Sha256::digest(secret);
        Self(digest.into())
    }

    pub(crate) fn from_token(token: &str) -> Option<Self> {
        if token.len() != SESSION_TOKEN_LENGTH {
            return None;
        }

        // The checked decoder requires its conservative maximum decoded size. A
        // canonical 43-byte token still has to decode to exactly 32 bytes.
        let mut secret = Zeroizing::new([0_u8; SESSION_DECODE_BUFFER_BYTES]);
        match URL_SAFE_NO_PAD.decode_slice(token.as_bytes(), secret.as_mut()) {
            Ok(SESSION_SECRET_BYTES) => Some(Self::from_secret(&secret[..SESSION_SECRET_BYTES])),
            Ok(_) | Err(_) => None,
        }
    }
}

/// Account authority cached for one authenticated browser session.
#[derive(Debug, Clone)]
pub struct Session {
    pub user_id: Uuid,
    pub role_type: RoleType,
    pub user_name: Arc<str>,
    pub user_country: i32,
    pub user_language: i32,
    pub is_email_verified: bool,
    pub created_at: jiff::Timestamp,
    pub expires_at: jiff::Timestamp,
}

impl Session {
    pub fn is_unexpired_at(&self, now: jiff::Timestamp) -> bool {
        self.created_at <= now && self.expires_at > now
    }

    pub fn get_user_id(&self) -> Uuid {
        self.user_id
    }

    pub fn get_role_type(&self) -> RoleType {
        self.role_type
    }

    pub fn get_user_name(&self) -> &str {
        self.user_name.as_ref()
    }

    pub fn get_user_country(&self) -> i32 {
        self.user_country
    }

    pub fn get_user_language(&self) -> i32 {
        self.user_language
    }

    pub fn get_is_email_verified(&self) -> bool {
        self.is_email_verified
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use jiff::{SignedDuration, Timestamp};
    use uuid::Uuid;

    use super::{SESSION_SECRET_BYTES, SESSION_TOKEN_LENGTH, Session, SessionKey, SessionToken};
    use crate::features::accounts::domain::role::RoleType;

    fn session(created_at: jiff::Timestamp, expires_at: jiff::Timestamp) -> Session {
        Session {
            user_id: Uuid::new_v4(),
            role_type: RoleType::User,
            user_name: Arc::from("test-user"),
            user_country: 1,
            user_language: 1,
            is_email_verified: true,
            created_at,
            expires_at,
        }
    }

    #[test]
    fn active_window_is_required() {
        let now = Timestamp::now();
        let minute = SignedDuration::from_mins(1);
        assert!(session(now, now + minute).is_unexpired_at(now));
        assert!(!session(now - minute * 2, now).is_unexpired_at(now));
        assert!(!session(now + minute, now + minute * 2).is_unexpired_at(now));
    }

    #[test]
    fn token_round_trip_is_fixed_size_and_unpadded() {
        let secret = [0xA5; SESSION_SECRET_BYTES];
        let token = SessionToken::from_secret(&secret);

        assert_eq!(token.expose().len(), SESSION_TOKEN_LENGTH);
        assert!(!token.expose().contains('='));
        assert!(SessionKey::from_token(token.expose()) == Some(SessionKey::from_secret(&secret)));
        assert_eq!(format!("{token:?}"), "SessionToken([REDACTED])");
    }

    #[test]
    fn malformed_tokens_are_rejected() {
        assert!(SessionKey::from_token("too-short").is_none());
        assert!(SessionKey::from_token(&"!".repeat(SESSION_TOKEN_LENGTH)).is_none());
    }
}
