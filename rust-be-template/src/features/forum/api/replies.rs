use axum::{
    Extension, Json,
    extract::{Path, State},
    response::IntoResponse,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    dto::{
        requests::forum::{
            moderation::{ForumReplyModerationActionRequest, ModerateForumReplyRequest},
            replies::{CreateForumReplyRequest, UpdateForumReplyRequest},
            topics::DeleteForumContentRequest,
        },
        responses::{
            forum::{moderation::ForumModerationResponse, topics::ForumReplyMutationResponse},
            response_data::http_resp,
        },
    },
    errors::code_error::HandlerResponse,
    features::forum::{api::error::map_forum_error, domain::enums::ForumModerationAction},
    init::state::ServerState,
    routers::middleware::logging::RequestLogContext,
    util::time::now::tokio_now,
};

#[utoipa::path(post, path = "/api/forum/topics/{topic_id}/replies", tag = "forum", params(("topic_id" = Uuid, Path)), request_body = CreateForumReplyRequest, responses((status = 200, body = ForumReplyMutationResponse), (status = 401), (status = 409)))]
pub async fn create_forum_reply(
    Extension(user_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    Path(topic_id): Path<Uuid>,
    Json(request): Json<CreateForumReplyRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let receipt = state
        .forum_service()
        .create_reply(user_id, topic_id, request.body)
        .await
        .map_err(map_forum_error)?;
    Ok(http_resp(
        ForumReplyMutationResponse::from(receipt),
        (),
        start,
    ))
}

#[utoipa::path(patch, path = "/api/forum/replies/{reply_id}", tag = "forum", params(("reply_id" = Uuid, Path)), request_body = UpdateForumReplyRequest, responses((status = 200, body = ForumReplyMutationResponse), (status = 403), (status = 409)))]
pub async fn update_forum_reply(
    Extension(user_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    Path(reply_id): Path<Uuid>,
    Json(request): Json<UpdateForumReplyRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let receipt = state
        .forum_service()
        .update_reply(user_id, reply_id, request.body, request.expected_revision)
        .await
        .map_err(map_forum_error)?;
    Ok(http_resp(
        ForumReplyMutationResponse::from(receipt),
        (),
        start,
    ))
}

#[utoipa::path(delete, path = "/api/forum/replies/{reply_id}", tag = "forum", params(("reply_id" = Uuid, Path)), request_body = DeleteForumContentRequest, responses((status = 200, body = ForumReplyMutationResponse), (status = 403), (status = 409)))]
pub async fn delete_forum_reply(
    Extension(user_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    Path(reply_id): Path<Uuid>,
    Json(request): Json<DeleteForumContentRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let receipt = state
        .forum_service()
        .delete_reply(user_id, reply_id, request.expected_revision)
        .await
        .map_err(map_forum_error)?;
    Ok(http_resp(
        ForumReplyMutationResponse::from(receipt),
        (),
        start,
    ))
}

#[utoipa::path(post, path = "/api/forum/replies/{reply_id}/moderation", tag = "forum", params(("reply_id" = Uuid, Path)), request_body = ModerateForumReplyRequest, responses((status = 200, body = ForumModerationResponse), (status = 403), (status = 409)))]
pub async fn moderate_forum_reply(
    Extension(user_id): Extension<Uuid>,
    Extension(log): Extension<RequestLogContext>,
    State(state): State<Arc<ServerState>>,
    Path(reply_id): Path<Uuid>,
    Json(request): Json<ModerateForumReplyRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let action = match request.action {
        ForumReplyModerationActionRequest::Hide => ForumModerationAction::ReplyHidden,
        ForumReplyModerationActionRequest::Restore => ForumModerationAction::ReplyRestored,
    };
    let request_id = Uuid::parse_str(&log.request_id).ok();
    let receipt = state
        .forum_service()
        .moderate_reply(
            user_id,
            reply_id,
            action,
            request.reason,
            request.expected_revision,
            request_id,
        )
        .await
        .map_err(map_forum_error)?;
    Ok(http_resp(ForumModerationResponse::from(receipt), (), start))
}
