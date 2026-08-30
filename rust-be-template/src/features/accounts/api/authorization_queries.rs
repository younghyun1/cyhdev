//! Superuser-only authorization catalog and account reads.

use axum::{
    Extension,
    extract::{Query, State},
    response::IntoResponse,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    dto::{
        requests::admin::authorization_request::{AuthorizationPageQuery, AuthorizationUsersQuery},
        responses::{
            admin::authorization_response::{
                AuthorizationPermissionItem, AuthorizationPermissionsResponse,
                AuthorizationRoleItem, AuthorizationRolesResponse, AuthorizationUserItem,
                AuthorizationUsersResponse, RolePermissionItem, RolePermissionsResponse,
            },
            response_data::http_resp,
        },
    },
    errors::code_error::{CodeErrorResp, HandlerResponse},
    init::state::ServerState,
    util::time::now::tokio_now,
};

use super::authorization_error::map_authorization_error;

#[utoipa::path(
    get,
    path = "/api/admin/authorization/users",
    tag = "admin",
    params(AuthorizationUsersQuery),
    responses(
        (status = 200, description = "Active accounts and current exclusive roles", body = AuthorizationUsersResponse),
        (status = 400, description = "Invalid bounded query", body = CodeErrorResp),
        (status = 403, description = "Current database role is not Younghyun", body = CodeErrorResp)
    )
)]
pub async fn list_authorization_users(
    State(state): State<Arc<ServerState>>,
    Extension(actor_user_id): Extension<Uuid>,
    Query(query): Query<AuthorizationUsersQuery>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let page = state
        .account_service()
        .authorization_users(actor_user_id, query.search, query.after, query.limit)
        .await
        .map_err(map_authorization_error)?;
    Ok(http_resp(
        AuthorizationUsersResponse {
            users: page
                .items
                .into_iter()
                .map(AuthorizationUserItem::from)
                .collect(),
            next_cursor: page.next_cursor,
        },
        (),
        start,
    ))
}

#[utoipa::path(
    get,
    path = "/api/admin/authorization/roles",
    tag = "admin",
    responses(
        (status = 200, description = "Bounded role catalog", body = AuthorizationRolesResponse),
        (status = 403, description = "Current database role is not Younghyun", body = CodeErrorResp)
    )
)]
pub async fn list_authorization_roles(
    State(state): State<Arc<ServerState>>,
    Extension(actor_user_id): Extension<Uuid>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let roles = state
        .account_service()
        .authorization_roles(actor_user_id)
        .await
        .map_err(map_authorization_error)?;
    Ok(http_resp(
        AuthorizationRolesResponse {
            roles: roles.into_iter().map(AuthorizationRoleItem::from).collect(),
        },
        (),
        start,
    ))
}

#[utoipa::path(
    get,
    path = "/api/admin/authorization/permissions",
    tag = "admin",
    responses(
        (status = 200, description = "Bounded permission catalog", body = AuthorizationPermissionsResponse),
        (status = 403, description = "Current database role is not Younghyun", body = CodeErrorResp)
    )
)]
pub async fn list_authorization_permissions(
    State(state): State<Arc<ServerState>>,
    Extension(actor_user_id): Extension<Uuid>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let permissions = state
        .account_service()
        .authorization_permissions(actor_user_id)
        .await
        .map_err(map_authorization_error)?;
    Ok(http_resp(
        AuthorizationPermissionsResponse {
            permissions: permissions
                .into_iter()
                .map(AuthorizationPermissionItem::from)
                .collect(),
        },
        (),
        start,
    ))
}

#[utoipa::path(
    get,
    path = "/api/admin/authorization/role-permissions",
    tag = "admin",
    params(AuthorizationPageQuery),
    responses(
        (status = 200, description = "Keyset-paginated role-permission bindings", body = RolePermissionsResponse),
        (status = 400, description = "Invalid bounded query", body = CodeErrorResp),
        (status = 403, description = "Current database role is not Younghyun", body = CodeErrorResp)
    )
)]
pub async fn list_role_permissions(
    State(state): State<Arc<ServerState>>,
    Extension(actor_user_id): Extension<Uuid>,
    Query(query): Query<AuthorizationPageQuery>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let page = state
        .account_service()
        .authorization_role_permissions(actor_user_id, query.after, query.limit)
        .await
        .map_err(map_authorization_error)?;
    Ok(http_resp(
        RolePermissionsResponse {
            bindings: page
                .items
                .into_iter()
                .map(RolePermissionItem::from)
                .collect(),
            next_cursor: page.next_cursor,
        },
        (),
        start,
    ))
}
