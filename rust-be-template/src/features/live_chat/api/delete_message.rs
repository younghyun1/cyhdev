//! Superuser HTTP boundary for live-chat moderation.

use crate::{
    dto::responses::response_data::http_resp,
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    features::live_chat::error::LiveChatError,
    init::state::ServerState,
    util::time::now::tokio_now,
};
use axum::{
    Extension,
    extract::{Path, State},
    response::IntoResponse,
};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Serialize, utoipa::ToSchema)]
pub struct DeleteLiveChatMessageResponse {
    pub deleted_message_id: Uuid,
}

#[utoipa::path(
    delete,
    path = "/api/admin/live-chat/messages/{message_id}",
    tag = "live_chat",
    params(("message_id" = Uuid, Path, description = "Message to delete")),
    responses(
        (status = 200, description = "Message absent after deletion; repeated requests are idempotent", body = DeleteLiveChatMessageResponse),
        (status = 401, description = "Unauthenticated", body = CodeErrorResp),
        (status = 403, description = "Current superuser authority required", body = CodeErrorResp),
        (status = 500, description = "Persistence failure", body = CodeErrorResp)
    )
)]
pub async fn delete_live_chat_message(
    State(state): State<Arc<ServerState>>,
    Extension(actor): Extension<Uuid>,
    Path(message_id): Path<Uuid>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let service = state.live_chat_service();
    // Finish post-commit cache invalidation even if the HTTP client disconnects.
    tokio::spawn(async move { service.delete_message(actor, message_id).await })
        .await
        .map_err(|error| code_err(CodeError::JOIN_ERROR, error))?
        .map_err(|error| {
            let code = match &error {
                LiveChatError::Unauthorized => CodeError::UNAUTHORIZED_ACCESS,
                LiveChatError::Forbidden => CodeError::IS_NOT_SUPERUSER,
                LiveChatError::InvalidCursor => CodeError::INVALID_REQUEST,
                LiveChatError::Pool(_) => CodeError::POOL_ERROR,
                LiveChatError::Database(_) => CodeError::DB_QUERY_ERROR,
            };
            code_err(code, error)
        })?;
    Ok(http_resp(
        DeleteLiveChatMessageResponse {
            deleted_message_id: message_id,
        },
        (),
        start,
    ))
}
