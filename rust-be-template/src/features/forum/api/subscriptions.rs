use std::sync::Arc;
use axum::{Extension, extract::{Path, State}, response::IntoResponse};
use uuid::Uuid;

use crate::{dto::responses::{forum::topics::ForumSubscriptionResponse, response_data::http_resp}, errors::code_error::HandlerResponse,
    features::forum::api::error::map_forum_error, init::state::ServerState, util::time::now::tokio_now};

#[utoipa::path(post, path = "/api/forum/topics/{topic_id}/subscription", tag = "forum", params(("topic_id" = Uuid, Path)), responses((status = 200, body = ForumSubscriptionResponse), (status = 409)))]
pub async fn subscribe_forum_topic(Extension(user_id): Extension<Uuid>, State(state): State<Arc<ServerState>>, Path(topic_id): Path<Uuid>) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let subscribed = state.forum_service().set_subscription(user_id, topic_id, true).await.map_err(map_forum_error)?;
    Ok(http_resp(ForumSubscriptionResponse { topic_id, subscribed }, (), start))
}

#[utoipa::path(delete, path = "/api/forum/topics/{topic_id}/subscription", tag = "forum", params(("topic_id" = Uuid, Path)), responses((status = 200, body = ForumSubscriptionResponse)))]
pub async fn unsubscribe_forum_topic(Extension(user_id): Extension<Uuid>, State(state): State<Arc<ServerState>>, Path(topic_id): Path<Uuid>) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let subscribed = state.forum_service().set_subscription(user_id, topic_id, false).await.map_err(map_forum_error)?;
    Ok(http_resp(ForumSubscriptionResponse { topic_id, subscribed }, (), start))
}
