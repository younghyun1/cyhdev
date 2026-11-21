//! Accounts allowed to read the archive.

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Most accounts the list may hold. The whole list is kept in memory and
/// shown on one page, and it is meant for people the owner knows.
pub const MAX_READERS: usize = 500;

/// One listed account, with what an administrator needs to recognize it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveReader {
    pub user_id: Uuid,
    pub user_name: String,
    pub user_email: String,
    pub granted_at: DateTime<Utc>,
}
