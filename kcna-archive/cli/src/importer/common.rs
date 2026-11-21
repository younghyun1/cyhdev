//! Helpers shared by the source importers.

use sha2::{Digest, Sha256};

use crate::{
    database::domain::CanonicalUrl,
    error::{Error, Result},
};

/// Validates a URL at the database boundary.
pub fn validated_url(value: &str) -> Result<CanonicalUrl> {
    CanonicalUrl::try_new(value).map_err(|error| Error::InvalidRecord(error.to_string()))
}

/// Digest of the raw response bytes.
pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

/// Digest of the title and body reduced to their letters and digits in lower
/// case. Sources format the same article differently (paragraph breaks,
/// spacing, punctuation), so only this form is equal across copies.
pub fn content_sha256(title: &str, body: Option<&str>) -> [u8; 32] {
    fn update(digest: &mut Sha256, text: &str) {
        let mut buffer = [0_u8; 4];
        for character in text.chars().filter(|character| character.is_alphanumeric()) {
            for lower in character.to_lowercase() {
                digest.update(lower.encode_utf8(&mut buffer).as_bytes());
            }
        }
    }
    let mut digest = Sha256::new();
    update(&mut digest, title);
    if let Some(body) = body.filter(|body| !body.is_empty()) {
        // The separator is outside the hashed alphabet, so the title and the
        // body cannot run into each other.
        digest.update(b"\n");
        update(&mut digest, body);
    }
    digest.finalize().into()
}

/// Worker count for CPU-bound page parsing: one per core, within a fixed
/// range so a small machine still overlaps file reads and a large one does not
/// queue hundreds of megabytes of raw pages.
pub fn parse_parallelism() -> usize {
    std::thread::available_parallelism()
        .map_or(4, std::num::NonZero::get)
        .clamp(2, 32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_digest_separates_title_from_body() {
        assert_ne!(
            content_sha256("ab", Some("c")),
            content_sha256("a", Some("bc"))
        );
        assert_eq!(content_sha256("a", None), content_sha256("a", Some("")));
        assert_ne!(content_sha256("a", None), content_sha256("a", Some("b")));
    }

    #[test]
    fn content_digest_ignores_formatting_but_not_wording() {
        assert_eq!(
            content_sha256("Peace Talks", Some("First.\n\nSecond paragraph!")),
            content_sha256("peace talks.", Some("First. Second  paragraph"))
        );
        assert_ne!(
            content_sha256("Greetings", Some("From A")),
            content_sha256("Greetings", Some("From B"))
        );
    }

    #[test]
    fn parallelism_stays_in_range() {
        assert!((2..=32).contains(&parse_parallelism()));
    }
}
