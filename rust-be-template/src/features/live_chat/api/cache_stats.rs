use std::sync::Arc;

use axum::{extract::State, response::IntoResponse};

use crate::{
    dto::responses::{
        live_chat::live_chat_cache_stats_response::LiveChatCacheStatsResponse,
        response_data::http_resp,
    },
    errors::code_error::{CodeErrorResp, HandlerResponse},
    init::state::ServerState,
    util::time::now::tokio_now,
};

/// Operational cache metrics for superusers. Served from the superuser router,
/// so connection counts and cache pressure are not public reconnaissance data.
#[utoipa::path(
    get,
    path = "/api/admin/live-chat/cache-stats",
    tag = "live_chat",
    responses(
        (status = 200, description = "Live chat cache stats", body = LiveChatCacheStatsResponse),
        (status = 401, description = "Unauthenticated", body = CodeErrorResp),
        (status = 403, description = "Current superuser authority required", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn get_live_chat_cache_stats(
    State(state): State<Arc<ServerState>>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let stats = state.live_chat_service().cache.stats().await;
    Ok(http_resp(
        LiveChatCacheStatsResponse::from(stats),
        (),
        start,
    ))
}

#[cfg(test)]
mod tests {
    use crate::docs::ApiDoc;
    use utoipa::OpenApi;

    #[test]
    fn cache_stats_are_documented_only_as_a_superuser_route() -> Result<(), serde_json::Error> {
        let document = serde_json::to_value(ApiDoc::openapi())?;
        assert!(
            document["paths"]
                .get("/api/live-chat/cache-stats")
                .is_none()
        );
        let responses = &document["paths"]["/api/admin/live-chat/cache-stats"]["get"]["responses"];
        assert!(responses.get("401").is_some());
        assert!(responses.get("403").is_some());
        Ok(())
    }
}
