//! Wires the archive service to its PostgreSQL-backed reader list.

use std::sync::Arc;

use diesel_async::{AsyncPgConnection, pooled_connection::bb8::Pool};

use super::{
    repository::access::ReaderRepository,
    service::{access::ReaderStore, archive_service::KcnaArchiveService},
};

/// Starts the archive service; the snapshot loads in the background.
pub async fn compose_kcna_archive(pool: Pool<AsyncPgConnection>) -> Arc<KcnaArchiveService> {
    let store: Arc<dyn ReaderStore> = Arc::new(ReaderRepository::new(pool));
    Arc::new(KcnaArchiveService::start_from_environment(store).await)
}
