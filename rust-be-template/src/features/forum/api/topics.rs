use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    dto::{
        requests::forum::{
            moderation::{ForumTopicModerationActionRequest, ModerateForumTopicRequest},
            pagination::{ForumTopicDetailQuery, ForumTopicListQuery},
            topics::{CreateForumTopicRequest, DeleteForumContentRequest, UpdateForumTopicRequest},
        },
        responses::{
            forum::{
                moderation::ForumModerationResponse,
                topics::{
                    ForumTopicDetailResponse, ForumTopicListResponse, ForumTopicMutationResponse,
                },
            },
            response_data::http_resp,
        },
    },
    errors::code_error::HandlerResponse,
    features::forum::{api::error::map_forum_error, domain::enums::ForumModerationAction},
    init::state::ServerState,
    routers::middleware::{is_logged_in::AuthStatus, logging::RequestLogContext},
    util::time::now::tokio_now,
};

#[utoipa::path(get, path = "/api/forum/topics", tag = "forum", params(ForumTopicListQuery), responses((status = 200, body = ForumTopicListResponse)))]
pub async fn list_forum_topics(
    State(state): State<Arc<ServerState>>,
    Query(query): Query<ForumTopicListQuery>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let page = state
        .forum_service()
        .topics(
            query.search,
            query.before_pinned,
            query.before_activity_at,
            query.before_topic_id,
            query.limit,
        )
        .await
        .map_err(map_forum_error)?;
    Ok(http_resp(
        ForumTopicListResponse {
            topics: page.items.into_iter().map(Into::into).collect(),
            next_cursor: page.next_cursor.map(Into::into),
        },
        (),
        start,
    ))
}

#[utoipa::path(get, path = "/api/forum/topics/{topic_id}", tag = "forum", params(("topic_id" = Uuid, Path), ForumTopicDetailQuery), responses((status = 200, body = ForumTopicDetailResponse), (status = 404)))]
pub async fn get_forum_topic(
    Extension(auth): Extension<AuthStatus>,
    State(state): State<Arc<ServerState>>,
    Path(topic_id): Path<Uuid>,
    Query(query): Query<ForumTopicDetailQuery>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let viewer = match auth {
        AuthStatus::LoggedIn(user_id) => Some(user_id),
        AuthStatus::LoggedOut => None,
    };
    let detail = state
        .forum_service()
        .topic(
            topic_id,
            viewer,
            query.after_reply_created_at,
            query.after_reply_id,
            query.reply_limit,
        )
        .await
        .map_err(map_forum_error)?;
    Ok(http_resp(
        ForumTopicDetailResponse {
            topic: detail.topic.into(),
            replies: detail.replies.items.into_iter().map(Into::into).collect(),
            next_reply_cursor: detail.replies.next_cursor.map(Into::into),
            is_subscribed: detail.is_subscribed,
        },
        (),
        start,
    ))
}

#[utoipa::path(post, path = "/api/forum/topics", tag = "forum", request_body = CreateForumTopicRequest, responses((status = 200, body = ForumTopicMutationResponse), (status = 400), (status = 401)))]
pub async fn create_forum_topic(
    Extension(user_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    Json(request): Json<CreateForumTopicRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let receipt = state
        .forum_service()
        .create_topic(user_id, request.title, request.body)
        .await
        .map_err(map_forum_error)?;
    Ok(http_resp(
        ForumTopicMutationResponse::from(receipt),
        (),
        start,
    ))
}

#[utoipa::path(patch, path = "/api/forum/topics/{topic_id}", tag = "forum", params(("topic_id" = Uuid, Path)), request_body = UpdateForumTopicRequest, responses((status = 200, body = ForumTopicMutationResponse), (status = 403), (status = 409)))]
pub async fn update_forum_topic(
    Extension(user_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    Path(topic_id): Path<Uuid>,
    Json(request): Json<UpdateForumTopicRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let receipt = state
        .forum_service()
        .update_topic(
            user_id,
            topic_id,
            request.title,
            request.body,
            request.expected_revision,
        )
        .await
        .map_err(map_forum_error)?;
    Ok(http_resp(
        ForumTopicMutationResponse::from(receipt),
        (),
        start,
    ))
}

#[utoipa::path(delete, path = "/api/forum/topics/{topic_id}", tag = "forum", params(("topic_id" = Uuid, Path)), request_body = DeleteForumContentRequest, responses((status = 200, body = ForumTopicMutationResponse), (status = 403), (status = 409)))]
pub async fn delete_forum_topic(
    Extension(user_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    Path(topic_id): Path<Uuid>,
    Json(request): Json<DeleteForumContentRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let receipt = state
        .forum_service()
        .delete_topic(user_id, topic_id, request.expected_revision)
        .await
        .map_err(map_forum_error)?;
    Ok(http_resp(
        ForumTopicMutationResponse::from(receipt),
        (),
        start,
    ))
}

#[utoipa::path(post, path = "/api/forum/topics/{topic_id}/moderation", tag = "forum", params(("topic_id" = Uuid, Path)), request_body = ModerateForumTopicRequest, responses((status = 200, body = ForumModerationResponse), (status = 403), (status = 409)))]
pub async fn moderate_forum_topic(
    Extension(user_id): Extension<Uuid>,
    Extension(log): Extension<RequestLogContext>,
    State(state): State<Arc<ServerState>>,
    Path(topic_id): Path<Uuid>,
    Json(request): Json<ModerateForumTopicRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let action = match request.action {
        ForumTopicModerationActionRequest::Hide => ForumModerationAction::TopicHidden,
        ForumTopicModerationActionRequest::Restore => ForumModerationAction::TopicRestored,
        ForumTopicModerationActionRequest::Lock => ForumModerationAction::TopicLocked,
        ForumTopicModerationActionRequest::Unlock => ForumModerationAction::TopicUnlocked,
        ForumTopicModerationActionRequest::Pin => ForumModerationAction::TopicPinned,
        ForumTopicModerationActionRequest::Unpin => ForumModerationAction::TopicUnpinned,
    };
    let request_id = Uuid::parse_str(&log.request_id).ok();
    let receipt = state
        .forum_service()
        .moderate_topic(
            user_id,
            topic_id,
            action,
            request.reason,
            request.expected_revision,
            request_id,
        )
        .await
        .map_err(map_forum_error)?;
    Ok(http_resp(ForumModerationResponse::from(receipt), (), start))
}
