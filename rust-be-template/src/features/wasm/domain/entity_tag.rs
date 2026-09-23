//! Content validators for revalidating WebAssembly bundle downloads.
//!
//! Module URLs are stable while their bytes change, so responses must be
//! revalidated. A strong entity tag derived from the stored bytes lets a
//! client that already holds them receive `304 Not Modified` instead of the
//! bundle, which can be up to 50 MiB.

use sha2::{Digest, Sha256};

/// Hex characters kept from the SHA-256 digest: 128 bits identify a bundle.
const DIGEST_HEX_CHARS: usize = 32;
/// Tags inspected from one `If-None-Match` header; further tags are ignored.
pub const MAX_IF_NONE_MATCH_TAGS: usize = 32;

/// Truncated SHA-256 of the stored (gzip) bundle bytes, as lowercase hex.
pub fn bundle_digest(stored_bytes: &[u8]) -> String {
    let digest = Sha256::digest(stored_bytes);
    let mut hex = String::with_capacity(DIGEST_HEX_CHARS);
    for byte in digest.iter().take(DIGEST_HEX_CHARS / 2) {
        hex.push(char::from(HEX[usize::from(byte >> 4)]));
        hex.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    hex
}

const HEX: &[u8; 16] = b"0123456789abcdef";

/// Opaque entity tag (without quotes) for one representation of a bundle.
/// The gzip and identity encodings are different bytes, so each gets its own
/// strong tag.
pub fn representation_tag(digest: &str, gzip: bool) -> String {
    if gzip {
        format!("sha256-{digest}-gz")
    } else {
        format!("sha256-{digest}")
    }
}

/// A parsed `If-None-Match` precondition.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum IfNoneMatch {
    #[default]
    Absent,
    Any,
    /// Opaque tags with quotes and any weak prefix removed.
    Tags(Vec<String>),
}

impl IfNoneMatch {
    /// `If-None-Match` uses weak comparison, so `W/"x"` matches tag `x`.
    pub fn matches(&self, opaque_tag: &str) -> bool {
        match self {
            Self::Absent => false,
            Self::Any => true,
            Self::Tags(tags) => tags.iter().any(|tag| tag == opaque_tag),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{IfNoneMatch, bundle_digest, representation_tag};

    #[test]
    fn digest_is_stable_truncated_hex() {
        let digest = bundle_digest(b"bundle");
        assert_eq!(digest.len(), 32);
        assert!(
            digest
                .chars()
                .all(|character| character.is_ascii_hexdigit())
        );
        assert_eq!(digest, bundle_digest(b"bundle"));
        assert_ne!(digest, bundle_digest(b"bundle2"));
    }

    #[test]
    fn encodings_get_distinct_tags_and_matching_is_exact() {
        let gzip = representation_tag("abc", true);
        let identity = representation_tag("abc", false);
        assert_ne!(gzip, identity);
        let condition = IfNoneMatch::Tags(vec![identity.clone()]);
        assert!(condition.matches(&identity));
        assert!(!condition.matches(&gzip));
        assert!(IfNoneMatch::Any.matches(&gzip));
        assert!(!IfNoneMatch::Absent.matches(&gzip));
    }
}
