use serde_derive::Serialize;
use utoipa::ToSchema;

/// Counts returned after deleting photograph records and their stored objects.
#[derive(Debug, Serialize, ToSchema)]
pub struct DeletePhotographsResponse {
    pub deleted_count: usize,
    pub s3_deleted_count: usize,
}
