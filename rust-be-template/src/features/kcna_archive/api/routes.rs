//! Route assembly. Reading is for superusers and the accounts on the reader
//! list; the snapshot's status, its reload, and the list are for superusers. The
//! composition root adds the session check to the first group and the
//! superuser and trusted-origin guards to the second.

use std::sync::Arc;

use axum::{
    Extension, Router,
    extract::DefaultBodyLimit,
    middleware::from_fn,
    routing::{delete, get, post},
};

use super::{
    access::{
        kcna_archive_access, kcna_archive_grant_reader, kcna_archive_readers,
        kcna_archive_revoke_reader,
    },
    document::kcna_archive_document,
    export::kcna_archive_export,
    guard::require_archive_reader,
    search::{kcna_archive_histogram, kcna_archive_search, kcna_archive_trends},
    status::{kcna_archive_reload, kcna_archive_status},
};
use crate::init::state::ServerState;

/// Largest accepted request body; reads take parameters in the query string
/// and the one body is a single account identifier.
const MAX_BODY_BYTES: usize = 1024;

/// Reader routes with the service attached.
pub fn reader_router(state: &Arc<ServerState>) -> Router<Arc<ServerState>> {
    reader_routes().layer(Extension(state.kcna_archive_service()))
}

/// Administrator routes with the service attached.
pub fn admin_router(state: &Arc<ServerState>) -> Router<Arc<ServerState>> {
    admin_routes().layer(Extension(state.kcna_archive_service()))
}

/// Reader route table, generic over the state so tests can mount it alone.
/// The gate covers the routes above it; the access route answers every
/// verified account, because it is how the site learns whether to show the
/// archive.
pub(super) fn reader_routes<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/api/kcna-archive/search", get(kcna_archive_search))
        .route("/api/kcna-archive/histogram", get(kcna_archive_histogram))
        .route("/api/kcna-archive/trends", get(kcna_archive_trends))
        .route(
            "/api/kcna-archive/documents/{document_id}",
            get(kcna_archive_document),
        )
        .route("/api/kcna-archive/export", get(kcna_archive_export))
        .route_layer(from_fn(require_archive_reader))
        .route("/api/kcna-archive/access", get(kcna_archive_access))
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
}

/// Administrator route table, generic over the state like the reader table.
pub(super) fn admin_routes<S: Clone + Send + Sync + 'static>() -> Router<S> {
    Router::new()
        .route("/api/admin/kcna-archive/status", get(kcna_archive_status))
        .route("/api/admin/kcna-archive/reload", post(kcna_archive_reload))
        .route(
            "/api/admin/kcna-archive/readers",
            get(kcna_archive_readers).post(kcna_archive_grant_reader),
        )
        .route(
            "/api/admin/kcna-archive/readers/{user_id}",
            delete(kcna_archive_revoke_reader),
        )
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
}
