use std::sync::Arc;
use axum::{Extension, extract::{Path, Query, State}, response::IntoResponse};
use uuid::Uuid;

use crate::{
    dto::{requests::forum::pagination::ForumNotificationListQuery, responses::{forum::notifications::{ForumNotificationListResponse, ForumNotificationReadResponse}, response_data::http_resp}},
    errors::code_error::HandlerResponse, features::forum::api::error::map_forum_error,
    init::state::ServerState, util::time::now::tokio_now,
};

#[utoipa::path(get, path = "/api/forum/notifications", tag = "forum", params(ForumNotificationListQuery), responses((status = 200, body = ForumNotificationListResponse), (status = 401)))]
pub async fn list_forum_notifications(
    Extension(user_id): Extension<Uuid>, State(state): State<Arc<ServerState>>, Query(query): Query<ForumNotificationListQuery>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let page = state.forum_service().notifications(user_id, query.before_created_at, query.before_notification_id, query.limit).await.map_err(map_forum_error)?;
    Ok(http_resp(ForumNotificationListResponse { notifications: page.items.into_iter().map(Into::into).collect(), next_cursor: page.next_cursor.map(Into::into) }, (), start))
}

#[utoipa::path(post, path = "/api/forum/notifications/{notification_id}/read", tag = "forum", params(("notification_id" = Uuid, Path)), responses((status = 200, body = ForumNotificationReadResponse), (status = 404)))]
pub async fn mark_forum_notification_read(
    Extension(user_id): Extension<Uuid>, State(state): State<Arc<ServerState>>, Path(notification_id): Path<Uuid>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let read_at = state.forum_service().mark_notification_read(user_id, notification_id).await.map_err(map_forum_error)?;
    Ok(http_resp(ForumNotificationReadResponse { notification_id, read_at }, (), start))
}
