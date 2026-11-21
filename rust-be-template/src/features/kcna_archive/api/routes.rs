//! Route assembly. The composition root merges these inside the superuser and
//! trusted-origin guards; handlers never check the role themselves.

use std::sync::Arc;

use axum::{
    Extension, Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};

use super::{
    document::kcna_archive_document,
    export::kcna_archive_export,
    search::{kcna_archive_histogram, kcna_archive_search, kcna_archive_trends},
    status::{kcna_archive_reload, kcna_archive_status},
};
use crate::init::state::ServerState;

/// Largest accepted request body; every archive route takes parameters in the query string.
const MAX_BODY_BYTES: usize = 1024;

/// Archive routes with the service attached.
pub fn router(state: &Arc<ServerState>) -> Router<Arc<ServerState>> {
    routes().layer(Extension(state.kcna_archive_service()))
}

/// Route table, generic over the state so tests can mount it alone.
pub(super) fn routes<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/api/admin/kcna-archive/search", get(kcna_archive_search))
        .route(
            "/api/admin/kcna-archive/histogram",
            get(kcna_archive_histogram),
        )
        .route("/api/admin/kcna-archive/trends", get(kcna_archive_trends))
        .route("/api/admin/kcna-archive/status", get(kcna_archive_status))
        .route(
            "/api/admin/kcna-archive/documents/{document_id}",
            get(kcna_archive_document),
        )
        .route("/api/admin/kcna-archive/export", get(kcna_archive_export))
        .route("/api/admin/kcna-archive/reload", post(kcna_archive_reload))
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
}
