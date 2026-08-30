use std::sync::Arc;

use axum::{extract::State, response::IntoResponse};

use crate::{
    dto::responses::{
        admin::sync_i18n_cache_response::SyncI18nCacheResponse, response_data::http_resp,
    },
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    init::state::ServerState,
    util::time::now::tokio_now,
};

#[utoipa::path(post, path = "/api/admin/sync-i18n-cache", tag = "admin", responses(
    (status = 200, description = "i18n cache synchronized", body = SyncI18nCacheResponse),
    (status = 500, description = "Internal server error", body = CodeErrorResp)
))]
pub async fn sync_i18n_cache(
    State(state): State<Arc<ServerState>>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let service = state.i18n_service();
    service
        .synchronize_file_sources()
        .await
        .map_err(|error| code_err(CodeError::COULD_NOT_SYNC_18N_CACHE, error))?;
    let num_rows = service
        .synchronize_cache()
        .await
        .map_err(|error| code_err(CodeError::COULD_NOT_SYNC_18N_CACHE, error))?;
    Ok(http_resp(SyncI18nCacheResponse { num_rows }, (), start))
}
