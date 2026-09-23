//! Periodic flush of buffered blog view counts to the database.

use std::sync::Arc;

use tracing::error;

use crate::init::state::ServerState;

pub async fn flush_blog_views(state: Arc<ServerState>) {
    if let Err(error) = state.blog_service().flush_views().await {
        error!(error = %error, "Failed to flush blog view counts");
    }
}
