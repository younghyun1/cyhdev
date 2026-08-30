//! Privacy-safe, keyset-paginated authorization audit endpoint.

use std::sync::Arc;

use axum::{
    Extension,
    extract::{Query, State},
    response::IntoResponse,
};
use uuid::Uuid;

use crate::{
    dto::{
        requests::admin::authorization_request::AuthorizationAuditQuery,
        responses::{
            admin::authorization_response::{
                AuthorizationAuditCursorItem, AuthorizationAuditItem, AuthorizationAuditResponse,
            },
            response_data::http_resp,
        },
    },
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    features::accounts::domain::authorization::AuthorizationAuditCursor,
    init::state::ServerState,
    util::time::now::tokio_now,
};

use super::authorization_error::map_authorization_error;

#[utoipa::path(
    get,
    path = "/api/admin/authorization/audit",
    tag = "admin",
    params(AuthorizationAuditQuery),
    responses(
        (status = 200, description = "Append-only authorization audit history", body = AuthorizationAuditResponse),
        (status = 400, description = "Invalid keyset cursor or page size", body = CodeErrorResp),
        (status = 403, description = "Current database role is not Younghyun", body = CodeErrorResp)
    )
)]
pub async fn list_authorization_audit(
    State(state): State<Arc<ServerState>>,
    Extension(actor_user_id): Extension<Uuid>,
    Query(query): Query<AuthorizationAuditQuery>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let before = audit_cursor(&query)?;
    let page = state
        .account_service()
        .authorization_audit_events(actor_user_id, before, query.limit)
        .await
        .map_err(map_authorization_error)?;
    Ok(http_resp(
        AuthorizationAuditResponse {
            events: page
                .items
                .into_iter()
                .map(AuthorizationAuditItem::from)
                .collect(),
            next_cursor: page.next_cursor.map(AuthorizationAuditCursorItem::from),
        },
        (),
        start,
    ))
}

fn audit_cursor(
    query: &AuthorizationAuditQuery,
) -> Result<Option<AuthorizationAuditCursor>, CodeErrorResp> {
    match (query.before_created_at, query.before_audit_event_id) {
        (Some(created_at), Some(audit_event_id)) => Ok(Some(AuthorizationAuditCursor {
            created_at,
            audit_event_id,
        })),
        (None, None) => Ok(None),
        (Some(_), None) | (None, Some(_)) => Err(code_err(
            CodeError::INVALID_REQUEST,
            "audit cursor timestamp and event ID must be supplied together",
        )),
    }
}
