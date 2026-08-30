//! Diesel rows private to audited authorization persistence.

use chrono::{DateTime, Utc};
use diesel::{Insertable, Queryable, Selectable};
use uuid::Uuid;

use crate::{
    features::accounts::domain::authorization::AuthorizationAuditKind,
    schema::authorization_audit_events,
};

#[derive(Queryable)]
pub(super) struct AuthorizationUserRow {
    pub(super) user_id: Uuid,
    pub(super) user_name: String,
    pub(super) user_email: String,
    pub(super) role_id: Uuid,
}

#[derive(Queryable)]
pub(super) struct RoleDefinitionRow {
    pub(super) role_id: Uuid,
    pub(super) role_description: Option<String>,
}

#[derive(Queryable)]
pub(super) struct PermissionDefinitionRow {
    pub(super) permission_id: Uuid,
    pub(super) permission_name: String,
    pub(super) permission_description: Option<String>,
}

#[derive(Queryable)]
pub(super) struct RolePermissionBindingRow {
    pub(super) role_permission_id: Uuid,
    pub(super) role_id: Uuid,
    pub(super) permission_id: Uuid,
    pub(super) permission_name: String,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = authorization_audit_events)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(super) struct AuthorizationAuditEventRow {
    pub(super) authorization_audit_event_id: Uuid,
    pub(super) authorization_audit_event_actor_user_id: Uuid,
    pub(super) authorization_audit_event_kind: AuthorizationAuditKind,
    pub(super) authorization_audit_event_target_user_id: Option<Uuid>,
    pub(super) authorization_audit_event_role_id: Uuid,
    pub(super) authorization_audit_event_role_name: String,
    pub(super) authorization_audit_event_permission_id: Option<Uuid>,
    pub(super) authorization_audit_event_permission_name: Option<String>,
    pub(super) authorization_audit_event_old_value: String,
    pub(super) authorization_audit_event_new_value: String,
    pub(super) authorization_audit_event_reason: String,
    pub(super) authorization_audit_event_request_id: Option<Uuid>,
    pub(super) authorization_audit_event_created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = authorization_audit_events)]
pub(super) struct NewAuthorizationAuditEventRecord<'a> {
    pub(super) authorization_audit_event_actor_user_id: Uuid,
    pub(super) authorization_audit_event_kind: AuthorizationAuditKind,
    pub(super) authorization_audit_event_target_user_id: Option<Uuid>,
    pub(super) authorization_audit_event_role_id: Uuid,
    pub(super) authorization_audit_event_role_name: &'static str,
    pub(super) authorization_audit_event_permission_id: Option<Uuid>,
    pub(super) authorization_audit_event_permission_name: Option<&'a str>,
    pub(super) authorization_audit_event_old_value: &'static str,
    pub(super) authorization_audit_event_new_value: &'static str,
    pub(super) authorization_audit_event_reason: &'a str,
    pub(super) authorization_audit_event_request_id: Option<Uuid>,
}
