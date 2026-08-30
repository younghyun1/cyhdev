//! Strongly typed public responses for authorization administration.

use chrono::{DateTime, Utc};
use serde_derive::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::features::accounts::domain::authorization::{
    AuthorizationAuditCursor, AuthorizationAuditEvent, AuthorizationAuditKind, AuthorizationUser,
    PermissionDefinition, RoleAssignmentReceipt, RoleDefinition, RolePermissionBinding,
    RolePermissionReceipt,
};

#[derive(Serialize, ToSchema)]
pub struct AuthorizationUsersResponse {
    pub users: Vec<AuthorizationUserItem>,
    pub next_cursor: Option<Uuid>,
}

#[derive(Serialize, ToSchema)]
pub struct AuthorizationUserItem {
    pub user_id: Uuid,
    pub user_name: String,
    pub user_email: String,
    pub role_id: Uuid,
    pub role_name: String,
}

impl From<AuthorizationUser> for AuthorizationUserItem {
    fn from(value: AuthorizationUser) -> Self {
        Self {
            user_id: value.user_id,
            user_name: value.user_name,
            user_email: value.user_email,
            role_id: value.role_type.id(),
            role_name: value.role_type.name().to_owned(),
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct AuthorizationRolesResponse {
    pub roles: Vec<AuthorizationRoleItem>,
}

#[derive(Serialize, ToSchema)]
pub struct AuthorizationRoleItem {
    pub role_id: Uuid,
    pub role_name: String,
    pub description: Option<String>,
}

impl From<RoleDefinition> for AuthorizationRoleItem {
    fn from(value: RoleDefinition) -> Self {
        Self {
            role_id: value.role_type.id(),
            role_name: value.role_type.name().to_owned(),
            description: value.description,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct AuthorizationPermissionsResponse {
    pub permissions: Vec<AuthorizationPermissionItem>,
}

#[derive(Serialize, ToSchema)]
pub struct AuthorizationPermissionItem {
    pub permission_id: Uuid,
    pub permission_name: String,
    pub description: Option<String>,
}

impl From<PermissionDefinition> for AuthorizationPermissionItem {
    fn from(value: PermissionDefinition) -> Self {
        Self {
            permission_id: value.permission_id,
            permission_name: value.permission_name.into_inner(),
            description: value.description,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct RolePermissionsResponse {
    pub bindings: Vec<RolePermissionItem>,
    pub next_cursor: Option<Uuid>,
}

#[derive(Serialize, ToSchema)]
pub struct RolePermissionItem {
    pub role_permission_id: Uuid,
    pub role_id: Uuid,
    pub role_name: String,
    pub permission_id: Uuid,
    pub permission_name: String,
}

impl From<RolePermissionBinding> for RolePermissionItem {
    fn from(value: RolePermissionBinding) -> Self {
        Self {
            role_permission_id: value.role_permission_id,
            role_id: value.role_type.id(),
            role_name: value.role_type.name().to_owned(),
            permission_id: value.permission_id,
            permission_name: value.permission_name.into_inner(),
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct AuthorizationAuditResponse {
    pub events: Vec<AuthorizationAuditItem>,
    pub next_cursor: Option<AuthorizationAuditCursorItem>,
}

#[derive(Serialize, ToSchema)]
pub struct AuthorizationAuditCursorItem {
    pub created_at: DateTime<Utc>,
    pub audit_event_id: Uuid,
}

impl From<AuthorizationAuditCursor> for AuthorizationAuditCursorItem {
    fn from(value: AuthorizationAuditCursor) -> Self {
        Self {
            created_at: value.created_at,
            audit_event_id: value.audit_event_id,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct AuthorizationAuditItem {
    pub audit_event_id: Uuid,
    pub actor_user_id: Uuid,
    pub actor_display_name: String,
    pub kind: String,
    pub target_user_id: Option<Uuid>,
    pub target_display_name: Option<String>,
    pub role_id: Uuid,
    pub role_name: String,
    pub permission_id: Option<Uuid>,
    pub permission_name: Option<String>,
    pub old_value: String,
    pub new_value: String,
    pub reason: String,
    pub request_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl From<AuthorizationAuditEvent> for AuthorizationAuditItem {
    fn from(value: AuthorizationAuditEvent) -> Self {
        Self {
            audit_event_id: value.audit_event_id,
            actor_user_id: value.actor_user_id,
            actor_display_name: value.actor_display_name,
            kind: audit_kind_name(value.kind).to_owned(),
            target_user_id: value.target_user_id,
            target_display_name: value.target_display_name,
            role_id: value.role_id,
            role_name: value.role_name,
            permission_id: value.permission_id,
            permission_name: value.permission_name.map(|name| name.into_inner()),
            old_value: value.old_value,
            new_value: value.new_value,
            reason: value.reason,
            request_id: value.request_id,
            created_at: value.created_at,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct RoleAssignmentResponse {
    pub audit_event_id: Uuid,
    pub user_id: Uuid,
    pub previous_role_id: Uuid,
    pub previous_role_name: String,
    pub role_id: Uuid,
    pub role_name: String,
}

impl From<RoleAssignmentReceipt> for RoleAssignmentResponse {
    fn from(value: RoleAssignmentReceipt) -> Self {
        Self {
            audit_event_id: value.audit_event_id,
            user_id: value.user_id,
            previous_role_id: value.previous_role.id(),
            previous_role_name: value.previous_role.name().to_owned(),
            role_id: value.role_type.id(),
            role_name: value.role_type.name().to_owned(),
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct RolePermissionChangeResponse {
    pub audit_event_id: Uuid,
    pub role_id: Uuid,
    pub role_name: String,
    pub permission_id: Uuid,
    pub permission_name: String,
    pub enabled: bool,
}

impl From<RolePermissionReceipt> for RolePermissionChangeResponse {
    fn from(value: RolePermissionReceipt) -> Self {
        Self {
            audit_event_id: value.audit_event_id,
            role_id: value.role_type.id(),
            role_name: value.role_type.name().to_owned(),
            permission_id: value.permission_id,
            permission_name: value.permission_name.into_inner(),
            enabled: value.enabled,
        }
    }
}

fn audit_kind_name(kind: AuthorizationAuditKind) -> &'static str {
    kind.as_str()
}
