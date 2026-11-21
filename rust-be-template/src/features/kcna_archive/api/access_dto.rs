//! Contract of the access endpoints: what a caller may do, and the reader
//! list an administrator reads and changes.

use serde_derive::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::features::kcna_archive::domain::reader::{ArchiveReader, MAX_READERS};

/// Whether the signed-in caller may read the archive right now.
#[derive(Clone, Copy, Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveAccessResponse {
    pub can_read: bool,
    /// True for a superuser while nobody else is listed, so the page can mark
    /// the archive as theirs alone. Always false for other accounts, which
    /// therefore learn nothing about the list.
    pub admin_only: bool,
}

/// One account on the reader list.
#[derive(Clone, Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveReaderItem {
    pub user_id: Uuid,
    pub user_name: String,
    pub user_email: String,
    /// RFC 3339 time the account was listed.
    pub granted_at: String,
}

impl From<ArchiveReader> for ArchiveReaderItem {
    fn from(reader: ArchiveReader) -> Self {
        Self {
            user_id: reader.user_id,
            user_name: reader.user_name,
            user_email: reader.user_email,
            granted_at: reader.granted_at.to_rfc3339(),
        }
    }
}

/// The reader list, newest grant first, and how many accounts it may hold.
#[derive(Clone, Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveReadersResponse {
    pub readers: Vec<ArchiveReaderItem>,
    pub limit: u32,
}

impl From<Vec<ArchiveReader>> for ArchiveReadersResponse {
    fn from(readers: Vec<ArchiveReader>) -> Self {
        Self {
            readers: readers.into_iter().map(ArchiveReaderItem::from).collect(),
            limit: MAX_READERS as u32,
        }
    }
}

/// The account to add to the reader list.
#[derive(Clone, Copy, Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArchiveReaderGrant {
    pub user_id: Uuid,
}
