pub struct PhotographDeleteReport {
    pub deleted_count: usize,
    pub s3_deleted_count: usize,
    pub cleanup_failure_count: usize,
    pub cleanup_remaining_count: usize,
    pub unresolved_cleanup_count: usize,
}
