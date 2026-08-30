//! Confirmed role and permission mutations with request-correlated audit.

use std::sync::Arc;

use axum::{Extension, Json, extract::{Path, State}, response::IntoResponse};
use uuid::Uuid;

use crate::{
    dto::{
        requests::admin::authorization_request::{AssignRoleRequest, SetRolePermissionRequest},
        responses::{
            admin::authorization_response::{
                RoleAssignmentResponse, RolePermissionChangeResponse,
            },
            response_data::http_resp,
        },
    },
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    init::state::ServerState,
    routers::middleware::logging::RequestLogContext,
    util::time::now::tokio_now,
};

use super::authorization_error::map_authorization_error;

#[utoipa::path(
    patch,
    path = "/api/admin/authorization/users/{user_id}/role",
    tag = "admin",
    request_body = AssignRoleRequest,
    params(("user_id" = Uuid, Path, description = "Account receiving the exclusive role")),
    responses(
        (status = 200, description = "Committed and audited role assignment", body = RoleAssignmentResponse),
        (status = 400, description = "Invalid reason or confirmation", body = CodeErrorResp),
        (status = 403, description = "Current database role is not Younghyun", body = CodeErrorResp),
        (status = 409, description = "Self-lockout, last-owner removal, or no-op", body = CodeErrorResp)
    )
)]
pub async fn assign_authorization_role(
    State(state): State<Arc<ServerState>>,
    Extension(actor_user_id): Extension<Uuid>,
    Extension(request_context): Extension<RequestLogContext>,
    Path(user_id): Path<Uuid>,
    Json(request): Json<AssignRoleRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    if !request.confirmed || request.confirmed_user_id != user_id {
        return Err(invalid_confirmation());
    }
    let receipt = state
        .account_service()
        .assign_role_as_administrator(
            actor_user_id,
            user_id,
            request.role_id,
            request.reason,
            request_id(&request_context),
        )
        .await
        .map_err(map_authorization_error)?;
    Ok(http_resp(RoleAssignmentResponse::from(receipt), (), start))
}

#[utoipa::path(
    patch,
    path = "/api/admin/authorization/roles/{role_id}/permissions/{permission_id}",
    tag = "admin",
    request_body = SetRolePermissionRequest,
    params(
        ("role_id" = Uuid, Path, description = "Role receiving or losing the permission"),
        ("permission_id" = Uuid, Path, description = "Permission binding to change")
    ),
    responses(
        (status = 200, description = "Committed and audited permission binding", body = RolePermissionChangeResponse),
        (status = 400, description = "Invalid reason or confirmation", body = CodeErrorResp),
        (status = 403, description = "Current database role is not Younghyun", body = CodeErrorResp),
        (status = 409, description = "Requested binding is already current", body = CodeErrorResp)
    )
)]
pub async fn set_authorization_role_permission(
    State(state): State<Arc<ServerState>>,
    Extension(actor_user_id): Extension<Uuid>,
    Extension(request_context): Extension<RequestLogContext>,
    Path((role_id, permission_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<SetRolePermissionRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    if !request.confirmed
        || request.confirmed_role_id != role_id
        || request.confirmed_permission_id != permission_id
    {
        return Err(invalid_confirmation());
    }
    let receipt = state
        .account_service()
        .set_role_permission_as_administrator(
            actor_user_id,
            role_id,
            permission_id,
            request.enabled,
            request.reason,
            request_id(&request_context),
        )
        .await
        .map_err(map_authorization_error)?;
    Ok(http_resp(
        RolePermissionChangeResponse::from(receipt),
        (),
        start,
    ))
}

fn invalid_confirmation() -> CodeErrorResp {
    code_err(
        CodeError::INVALID_REQUEST,
        "authorization change confirmation does not match the path",
    )
}

fn request_id(context: &RequestLogContext) -> Option<Uuid> {
    Uuid::parse_str(&context.request_id).ok()
}
