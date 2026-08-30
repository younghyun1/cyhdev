use std::sync::Arc;

use axum::{extract::{Query, State}, response::IntoResponse};

use crate::{
    dto::{
        requests::live_chat::get_live_chat_messages_request::GetLiveChatMessagesRequest,
        responses::{
            live_chat::{
                get_live_chat_messages_response::GetLiveChatMessagesResponse,
                live_chat_message_response::LiveChatMessageItem,
            },
            response_data::http_resp,
        },
    },
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    init::state::ServerState,
    util::time::now::tokio_now,
};

#[utoipa::path(
    get,
    path = "/api/live-chat/messages",
    tag = "live_chat",
    params(
        ("limit" = Option<usize>, Query, description = "Maximum number of messages"),
        ("before_message_id" = Option<uuid::Uuid>, Query, description = "Cursor message ID")
    ),
    responses(
        (status = 200, description = "Live chat messages", body = GetLiveChatMessagesResponse),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn get_live_chat_messages(
    State(state): State<Arc<ServerState>>,
    Query(request): Query<GetLiveChatMessagesRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let messages = state.live_chat_service().page_messages(request.before_message_id, request.limit).await
        .map_err(|error| {
            let code = match &error {
                super::super::error::LiveChatError::InvalidCursor => CodeError::INVALID_REQUEST,
                super::super::error::LiveChatError::Pool(_) => CodeError::POOL_ERROR,
                _ => CodeError::DB_QUERY_ERROR,
            };
            code_err(code, error)
        })?;
    let has_more = messages.len() == request.limit.clamp(1, 100);
    let next_before_message_id = messages.first().map(|message| message.live_chat_message_id);
    let items = messages.into_iter().map(LiveChatMessageItem::from).collect();
    Ok(http_resp(GetLiveChatMessagesResponse { items, next_before_message_id, has_more }, (), start))
}
