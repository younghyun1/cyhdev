//! Response contract of the status and reload endpoints.

use serde_derive::Serialize;
use utoipa::ToSchema;

use super::dto::{ArchiveLanguageFacet, ArchiveSourceFacet};

/// Where the in-memory snapshot is in its life cycle.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ArchiveServiceState {
    /// No snapshot exists on this server.
    NotInstalled,
    /// The first load is running.
    Loading,
    /// A snapshot is in memory and answering.
    Ready,
    /// The first load failed; reloading may repair it.
    Failed,
}

/// Corpus and index summary of the loaded snapshot.
#[derive(Clone, Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveSnapshotStatus {
    /// Documents stored in the collector's database when the snapshot was written.
    pub documents: u64,
    /// Of those, duplicates folded into another source's copy.
    pub copies: u64,
    /// Articles in the search index.
    pub indexed: u64,
    pub translation_groups: u64,
    pub oldest_date: Option<String>,
    pub newest_date: Option<String>,
    /// RFC 3339 time of the newest stored retrieval.
    pub last_retrieved_at: Option<String>,
    /// RFC 3339 time the snapshot was written.
    pub snapshot_built_at: String,
    /// Bytes of index held in memory.
    pub memory_bytes: Option<u64>,
    pub languages: Vec<ArchiveLanguageFacet>,
    pub sources: Vec<ArchiveSourceFacet>,
}

/// Service state, with the snapshot summary when one is loaded.
#[derive(Clone, Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveStatusResponse {
    pub state: ArchiveServiceState,
    /// A reload is running while the previous snapshot keeps serving.
    pub reloading: bool,
    /// The newest reload failed and the previous snapshot is still serving.
    pub last_reload_failed: bool,
    /// Present only when `state` is `ready`.
    pub snapshot: Option<ArchiveSnapshotStatus>,
}
