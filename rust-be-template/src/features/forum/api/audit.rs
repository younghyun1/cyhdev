use axum::{
    Extension,
    extract::{Query, State},
    response::IntoResponse,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    dto::{
        requests::forum::pagination::ForumModerationAuditListQuery,
        responses::{
            forum::moderation::ForumModerationAuditListResponse, response_data::http_resp,
        },
    },
    errors::code_error::HandlerResponse,
    features::forum::api::error::map_forum_error,
    init::state::ServerState,
    util::time::now::tokio_now,
};

#[utoipa::path(get, path = "/api/forum/moderation/audit", tag = "forum", params(ForumModerationAuditListQuery), responses((status = 200, body = ForumModerationAuditListResponse), (status = 403)))]
pub async fn list_forum_moderation_audit(
    Extension(user_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    Query(query): Query<ForumModerationAuditListQuery>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let page = state
        .forum_service()
        .moderation_audit(
            user_id,
            query.before_created_at,
            query.before_audit_id,
            query.limit,
        )
        .await
        .map_err(map_forum_error)?;
    Ok(http_resp(
        ForumModerationAuditListResponse {
            events: page.items.into_iter().map(Into::into).collect(),
            next_cursor: page.next_cursor.map(Into::into),
        },
        (),
        start,
    ))
}
