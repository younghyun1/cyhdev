use serde_derive::Serialize;
use utoipa::ToSchema;

/// Counts returned after deleting photograph records and their stored objects.
#[derive(Debug, Serialize, ToSchema)]
pub struct DeletePhotographsResponse {
    pub deleted_count: usize,
    pub s3_deleted_count: usize,
    pub cleanup_failure_count: usize,
    pub cleanup_remaining_count: usize,
    pub unresolved_cleanup_count: usize,
}

impl DeletePhotographsResponse {
    pub fn empty() -> Self {
        Self {
            deleted_count: 0,
            s3_deleted_count: 0,
            cleanup_failure_count: 0,
            cleanup_remaining_count: 0,
            unresolved_cleanup_count: 0,
        }
    }
}
