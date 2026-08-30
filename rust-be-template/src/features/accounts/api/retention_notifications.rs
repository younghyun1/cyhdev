//! Bounded superuser inspection and retry for retention notifications.

use std::sync::Arc;

use axum::{
    Extension,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use uuid::Uuid;

use crate::{
    dto::{
        requests::admin::retention_notification_request::RetentionNotificationStatusRequest,
        responses::{
            admin::retention_notification_response::{
                RetentionNotificationStatusItem, RetentionNotificationStatusResponse,
                RetryRetentionNotificationResponse,
            },
            response_data::http_resp_sensitive,
        },
    },
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    features::accounts::api::account_error::{AccountMutation, map_account_error},
    init::state::ServerState,
    util::time::now::tokio_now,
};

#[utoipa::path(
    get,
    path = "/api/admin/account-retention-notifications",
    tag = "admin",
    params(RetentionNotificationStatusRequest),
    responses(
        (status = 200, description = "Bounded retention-notification status page", body = RetentionNotificationStatusResponse),
        (status = 400, description = "Cursor fields are incomplete", body = CodeErrorResp),
        (status = 401, description = "Authentication required", body = CodeErrorResp),
        (status = 403, description = "Current database role is not superuser", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn retention_notification_status(
    Extension(requester_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    Query(request): Query<RetentionNotificationStatusRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let cursor = request.cursor().map_err(|_error| {
        code_err(
            CodeError::INVALID_REQUEST,
            "retention notification cursor fields must be provided together",
        )
    })?;
    let statuses = state
        .account_service()
        .retention_notification_status(requester_id, cursor, request.requested_limit())
        .await
        .map_err(|error| map_account_error(error, AccountMutation::Update))?;
    let next_cursor = statuses
        .last()
        .map(|status| (status.next_attempt_at, status.notification_id));
    let notifications = statuses
        .into_iter()
        .map(|status| RetentionNotificationStatusItem {
            notification_id: status.notification_id,
            user_id: status.user_id,
            stage: status.stage,
            scheduled_for: status.scheduled_for,
            next_attempt_at: status.next_attempt_at,
            attempt_count: status.attempt_count,
            claim_expires_at: status.claim_expires_at,
            sent_at: status.sent_at,
            cancelled_at: status.cancelled_at,
            last_error: status.last_error,
        })
        .collect();
    let (next_after_next_attempt_at, next_after_notification_id) = match next_cursor {
        Some((next_attempt_at, notification_id)) => (Some(next_attempt_at), Some(notification_id)),
        None => (None, None),
    };
    Ok(http_resp_sensitive(
        RetentionNotificationStatusResponse {
            notifications,
            next_after_next_attempt_at,
            next_after_notification_id,
        },
        (),
        start,
    ))
}

#[utoipa::path(
    post,
    path = "/api/admin/account-retention-notifications/{notification_id}/retry",
    tag = "admin",
    params(("notification_id" = Uuid, Path, description = "Retention notification identifier")),
    responses(
        (status = 200, description = "Unsent due notification queued for immediate retry", body = RetryRetentionNotificationResponse),
        (status = 401, description = "Authentication required", body = CodeErrorResp),
        (status = 403, description = "Current database role is not superuser", body = CodeErrorResp),
        (status = 409, description = "Notification was sent, cancelled, not due, or lost retained identity", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn retry_retention_notification(
    Extension(requester_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    Path(notification_id): Path<Uuid>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let receipt = state
        .account_service()
        .retry_retention_notification(requester_id, notification_id)
        .await
        .map_err(|error| map_account_error(error, AccountMutation::Update))?;
    Ok(http_resp_sensitive(
        RetryRetentionNotificationResponse {
            notification_id: receipt.notification_id,
            next_attempt_at: receipt.next_attempt_at,
        },
        (),
        start,
    ))
}
