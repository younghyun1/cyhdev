//! Strict forum input and pagination bounds.

use nutype::nutype;

pub const DEFAULT_TOPIC_PAGE_SIZE: u16 = 25;
pub const DEFAULT_REPLY_PAGE_SIZE: u16 = 50;
pub const DEFAULT_NOTIFICATION_PAGE_SIZE: u16 = 50;
pub const DEFAULT_AUDIT_PAGE_SIZE: u16 = 50;

#[nutype(
    sanitize(trim),
    validate(
        len_char_min = 3,
        len_char_max = 160,
        predicate = title_fits_bytes
    ),
    derive(Debug, Clone, PartialEq, Eq, AsRef, Display, TryFrom)
)]
pub struct ForumTitle(String);

#[nutype(
    sanitize(trim),
    validate(
        len_char_min = 1,
        len_char_max = 20000,
        predicate = body_fits_bytes
    ),
    derive(Debug, Clone, PartialEq, Eq, AsRef, Display, TryFrom)
)]
pub struct ForumBody(String);

#[nutype(
    sanitize(trim),
    validate(
        len_char_min = 8,
        len_char_max = 500,
        predicate = reason_fits_bytes
    ),
    derive(Debug, Clone, PartialEq, Eq, AsRef, Display, TryFrom)
)]
pub struct ForumModerationReason(String);

#[nutype(
    sanitize(trim),
    validate(
        len_char_min = 1,
        len_char_max = 128,
        predicate = valid_search
    ),
    derive(Debug, Clone, PartialEq, Eq, AsRef, Display, TryFrom)
)]
pub struct ForumSearch(String);

#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 100),
    derive(Debug, Clone, Copy, PartialEq, Eq, Into, TryFrom)
)]
pub struct ForumPageSize(u16);

fn title_fits_bytes(value: &str) -> bool {
    value.len() <= 512 && text_is_postgres_safe(value)
}

fn body_fits_bytes(value: &str) -> bool {
    value.len() <= 65_536 && text_is_postgres_safe(value)
}

fn reason_fits_bytes(value: &str) -> bool {
    value.len() <= 2_000 && text_is_postgres_safe(value)
}

fn valid_search(value: &str) -> bool {
    value.len() <= 512
        && text_is_postgres_safe(value)
        && value.split_whitespace().take(17).count() <= 16
}

fn text_is_postgres_safe(value: &str) -> bool {
    // PostgreSQL text rejects NUL at the protocol boundary. Reject it as a
    // client error instead of surfacing an avoidable database failure.
    !value.contains('\0')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn character_and_byte_limits_agree_with_database_checks() {
        assert!(ForumTitle::try_new("가".repeat(160)).is_ok());
        assert!(ForumTitle::try_new("🙂".repeat(160)).is_err());
        assert!(ForumBody::try_new("a".repeat(20_000)).is_ok());
        assert!(ForumBody::try_new("a".repeat(20_001)).is_err());
        assert!(ForumBody::try_new("🙂".repeat(16_384)).is_ok());
        assert!(ForumBody::try_new("🙂".repeat(16_385)).is_err());
        assert!(ForumModerationReason::try_new("short").is_err());
        assert!(ForumTitle::try_new("valid\0title").is_err());
        assert!(ForumBody::try_new("valid\0body").is_err());
        assert!(ForumModerationReason::try_new("valid reason\0text").is_err());
    }

    #[test]
    fn search_work_is_token_bounded() {
        assert!(ForumSearch::try_new("one two three").is_ok());
        assert!(
            ForumSearch::try_new((0..17).map(|_| "term").collect::<Vec<_>>().join(" ")).is_err()
        );
        assert!(ForumSearch::try_new("valid\0query").is_err());
    }
}
