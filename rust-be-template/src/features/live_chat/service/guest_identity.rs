//! Environment configuration for the keyed guest identity.

use tracing::{info, warn};

use super::super::domain::guest_identity::{GUEST_IDENTITY_MIN_SECRET_BYTES, GuestIdentityKey};

/// Optional secret that keeps guest keys and nicknames stable across restarts.
pub const LIVE_CHAT_GUEST_KEY_SECRET_ENV: &str = "LIVE_CHAT_GUEST_KEY_SECRET";

/// Load the guest identity key from `LIVE_CHAT_GUEST_KEY_SECRET`.
///
/// An absent or empty value selects a random per-process secret, so guest
/// identities change on restart but never expose an address. A value shorter
/// than [`GUEST_IDENTITY_MIN_SECRET_BYTES`] is rejected the same way: with only
/// 2^32 IPv4 inputs, a guessable secret would make every guest key reversible.
pub fn load_guest_identity_key() -> Result<GuestIdentityKey, getrandom::Error> {
    match std::env::var(LIVE_CHAT_GUEST_KEY_SECRET_ENV) {
        Ok(secret) => {
            let secret = secret.trim();
            if secret.len() >= GUEST_IDENTITY_MIN_SECRET_BYTES {
                return Ok(GuestIdentityKey::from_secret(secret.as_bytes()));
            }
            if secret.is_empty() {
                info!(
                    env_key = LIVE_CHAT_GUEST_KEY_SECRET_ENV,
                    "Live-chat guest identity secret is empty; using a random per-process secret"
                );
            } else {
                warn!(
                    env_key = LIVE_CHAT_GUEST_KEY_SECRET_ENV,
                    min_bytes = GUEST_IDENTITY_MIN_SECRET_BYTES,
                    "Live-chat guest identity secret is too short; using a random per-process secret"
                );
            }
        }
        Err(_) => info!(
            env_key = LIVE_CHAT_GUEST_KEY_SECRET_ENV,
            "Live-chat guest identity secret not configured; using a random per-process secret"
        ),
    }
    GuestIdentityKey::random()
}
