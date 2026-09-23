//! One-time email capabilities that PostgreSQL stores only as SHA-256 digests.
//!
//! Password-reset and email-verification links carry 256 random bits as 43 unpadded
//! base64url characters. The database keeps the SHA-256 of that text under a unique index,
//! so a leaked table or backup cannot be replayed as a working link.

use std::fmt;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

pub const CAPABILITY_SECRET_BYTES: usize = 32;
pub const CAPABILITY_TOKEN_LENGTH: usize = 43;
pub const CAPABILITY_DIGEST_BYTES: usize = 32;
/// Decoding needs room for the conservative maximum size of 43 base64 characters.
const CAPABILITY_DECODE_BUFFER_BYTES: usize = 33;

/// Raw capability placed only in the emailed link fragment.
pub struct CapabilityToken(Zeroizing<String>);

impl CapabilityToken {
    /// Draws a fresh capability from operating-system entropy together with its digest.
    pub fn generate() -> Result<(Self, CapabilityDigest), getrandom::Error> {
        let mut secret = Zeroizing::new([0_u8; CAPABILITY_SECRET_BYTES]);
        getrandom::fill(secret.as_mut())?;
        let token = Zeroizing::new(URL_SAFE_NO_PAD.encode(secret.as_ref()));
        let digest = CapabilityDigest::of_text(&token);
        Ok((Self(token), digest))
    }

    /// Exposes the token only for building an email link.
    pub fn expose(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Debug for CapabilityToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("CapabilityToken([REDACTED])")
    }
}

/// SHA-256 of a capability's text form; the only representation that is persisted.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct CapabilityDigest([u8; CAPABILITY_DIGEST_BYTES]);

impl CapabilityDigest {
    /// Digests a submitted token after checking its canonical 43-character form.
    ///
    /// Anything else, including the UUID links issued before digests were stored, is
    /// rejected before a database lookup.
    pub fn from_submitted(token: &str) -> Option<Self> {
        if token.len() != CAPABILITY_TOKEN_LENGTH {
            return None;
        }
        let mut decoded = Zeroizing::new([0_u8; CAPABILITY_DECODE_BUFFER_BYTES]);
        match URL_SAFE_NO_PAD.decode_slice(token.as_bytes(), decoded.as_mut()) {
            Ok(CAPABILITY_SECRET_BYTES) => Some(Self::of_text(token)),
            Ok(_) | Err(_) => None,
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    fn of_text(token: &str) -> Self {
        Self(Sha256::digest(token.as_bytes()).into())
    }
}

impl fmt::Debug for CapabilityDigest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("CapabilityDigest([REDACTED])")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_tokens_are_canonical_and_digest_to_their_submission() {
        let generated = CapabilityToken::generate();
        assert!(generated.is_ok());
        if let Ok((token, digest)) = generated {
            assert_eq!(token.expose().len(), CAPABILITY_TOKEN_LENGTH);
            assert!(!token.expose().contains('='));
            assert_eq!(
                CapabilityDigest::from_submitted(token.expose()),
                Some(digest)
            );
            assert_eq!(digest.as_bytes().len(), CAPABILITY_DIGEST_BYTES);
            assert_eq!(format!("{token:?}"), "CapabilityToken([REDACTED])");
        }
    }

    #[test]
    fn legacy_and_malformed_tokens_are_rejected() {
        assert!(CapabilityDigest::from_submitted("0190f1c0-7c4e-7b1a-8f00-0123456789ab").is_none());
        assert!(CapabilityDigest::from_submitted(&"!".repeat(CAPABILITY_TOKEN_LENGTH)).is_none());
        assert!(CapabilityDigest::from_submitted("").is_none());
        // The final character carries unused low bits; a non-canonical value is refused.
        let mut noncanonical = "A".repeat(CAPABILITY_TOKEN_LENGTH - 1);
        noncanonical.push('B');
        assert!(CapabilityDigest::from_submitted(&noncanonical).is_none());
    }

    #[test]
    fn digest_matches_sha256_of_the_token_text() {
        let token = "A".repeat(CAPABILITY_TOKEN_LENGTH);
        let expected: [u8; 32] = Sha256::digest(token.as_bytes()).into();
        assert_eq!(
            CapabilityDigest::from_submitted(&token).map(|digest| digest.0),
            Some(expected)
        );
    }
}
