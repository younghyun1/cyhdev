//! Validated-at-service-boundary inputs for authorization administration.

use chrono::{DateTime, Utc};
use serde_derive::Deserialize;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct AuthorizationUsersQuery {
    pub search: Option<String>,
    pub after: Option<Uuid>,
    pub limit: Option<u16>,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct AuthorizationPageQuery {
    pub after: Option<Uuid>,
    pub limit: Option<u16>,
}

#[derive(Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct AuthorizationAuditQuery {
    pub before_created_at: Option<DateTime<Utc>>,
    pub before_audit_event_id: Option<Uuid>,
    pub limit: Option<u16>,
}

#[derive(Deserialize, ToSchema)]
pub struct AssignRoleRequest {
    pub role_id: Uuid,
    pub reason: String,
    pub confirmed: bool,
    pub confirmed_user_id: Uuid,
}

#[derive(Deserialize, ToSchema)]
pub struct SetRolePermissionRequest {
    pub enabled: bool,
    pub reason: String,
    pub confirmed: bool,
    pub confirmed_role_id: Uuid,
    pub confirmed_permission_id: Uuid,
}
