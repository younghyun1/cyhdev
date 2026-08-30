use std::sync::Arc;

use axum::{Extension, extract::State, response::IntoResponse};
use uuid::Uuid;

use crate::{
    dto::responses::{
        admin::sync_i18n_cache_response::SyncI18nCacheResponse, response_data::http_resp,
    },
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    features::accounts::api::map_authorization_error,
    init::state::ServerState,
    util::time::now::tokio_now,
};

#[utoipa::path(post, path = "/api/admin/sync-i18n-cache", tag = "admin", responses(
    (status = 200, description = "i18n cache synchronized", body = SyncI18nCacheResponse),
    (status = 401, description = "Authentication required", body = CodeErrorResp),
    (status = 403, description = "Current database role is not Younghyun", body = CodeErrorResp),
    (status = 500, description = "Internal server error", body = CodeErrorResp)
))]
pub async fn sync_i18n_cache(
    Extension(requester_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let account_service = state.account_service();
    let authority_lease = account_service
        .acquire_current_younghyun_authority(requester_id)
        .await
        .map_err(map_authorization_error)?;
    let service = state.i18n_service();
    service
        .synchronize_file_sources()
        .await
        .map_err(|error| code_err(CodeError::COULD_NOT_SYNC_18N_CACHE, error))?;
    let num_rows = service
        .synchronize_cache()
        .await
        .map_err(|error| code_err(CodeError::COULD_NOT_SYNC_18N_CACHE, error))?;
    drop(authority_lease);
    Ok(http_resp(SyncI18nCacheResponse { num_rows }, (), start))
}

#[cfg(test)]
mod tests {
    use crate::docs::ApiDoc;
    use utoipa::OpenApi;

    #[test]
    fn documents_i18n_sync_auth_failures() -> Result<(), serde_json::Error> {
        let document = serde_json::to_value(ApiDoc::openapi())?;
        let responses = &document["paths"]["/api/admin/sync-i18n-cache"]["post"]["responses"];
        assert!(responses.get("401").is_some());
        assert!(responses.get("403").is_some());
        Ok(())
    }
}
