//! Serialized role and permission changes with append-only audit.
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    features::accounts::{
        authorization_error::AuthorizationError,
        domain::{
            authorization::{
                AuthorizationAuditKind, AuthorizationReason, PermissionName, RoleAssignmentReceipt,
                RolePermissionReceipt,
            },
            role::RoleType,
        },
        repository::{
            account_repository::AccountRepository,
            authorization_guard::{LockedYounghyun, lock_active_younghyun_authority},
            authorization_records::NewAuthorizationAuditEventRecord,
        },
    },
    schema::{authorization_audit_events, permissions, role_permissions, roles, user_roles, users},
};

impl AccountRepository {
    pub async fn assign_role_with_audit(
        &self,
        actor_user_id: Uuid,
        target_user_id: Uuid,
        role_type: RoleType,
        reason: &AuthorizationReason,
        request_id: Option<Uuid>,
    ) -> Result<RoleAssignmentReceipt, AuthorizationError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<RoleAssignmentReceipt, AuthorizationError, _>(async move |connection| {
                let (actor, owner_count) =
                    lock_active_younghyun_authority(connection, actor_user_id).await?;
                lock_known_role(connection, role_type).await?;
                let target = lock_role_target(connection, target_user_id).await?;
                if target.role_type == role_type {
                    return Err(AuthorizationError::NoChange);
                }
                if target.role_type == RoleType::Younghyun && role_type != RoleType::Younghyun {
                    if owner_count <= 1 {
                        return Err(AuthorizationError::LastActiveYounghyun);
                    }
                    if target_user_id == actor.user_id {
                        return Err(AuthorizationError::SelfLockout);
                    }
                }

                diesel::update(user_roles::table.filter(user_roles::user_id.eq(target_user_id)))
                    .set(user_roles::role_id.eq(role_type.id()))
                    .execute(&mut *connection)
                    .await?;

                let audit_event_id = insert_audit_event(
                    connection,
                    NewAuthorizationAuditEventRecord {
                        authorization_audit_event_actor_user_id: actor.user_id,
                        authorization_audit_event_kind: AuthorizationAuditKind::UserRoleAssigned
                            .into(),
                        authorization_audit_event_target_user_id: Some(target_user_id),
                        authorization_audit_event_role_id: role_type.id(),
                        authorization_audit_event_role_name: role_type.name(),
                        authorization_audit_event_permission_id: None,
                        authorization_audit_event_permission_name: None,
                        authorization_audit_event_old_value: target.role_type.name(),
                        authorization_audit_event_new_value: role_type.name(),
                        authorization_audit_event_reason: reason.as_ref(),
                        authorization_audit_event_request_id: request_id,
                    },
                )
                .await?;

                Ok(RoleAssignmentReceipt {
                    audit_event_id,
                    user_id: target_user_id,
                    previous_role: target.role_type,
                    role_type,
                })
            })
            .await
    }

    pub async fn set_role_permission_with_audit(
        &self,
        actor_user_id: Uuid,
        role_type: RoleType,
        permission_id: Uuid,
        enabled: bool,
        reason: &AuthorizationReason,
        request_id: Option<Uuid>,
    ) -> Result<RolePermissionReceipt, AuthorizationError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<RolePermissionReceipt, AuthorizationError, _>(async move |connection| {
                let (actor, _) = lock_active_younghyun_authority(connection, actor_user_id).await?;
                lock_known_role(connection, role_type).await?;
                let permission_name = lock_permission(connection, permission_id).await?;
                if role_type == RoleType::Younghyun && !enabled {
                    return Err(AuthorizationError::YounghyunPermissionProtected);
                }
                let existing = role_permissions::table
                    .filter(role_permissions::role_id.eq(role_type.id()))
                    .filter(role_permissions::permission_id.eq(permission_id))
                    .select(role_permissions::role_permission_id)
                    .for_update()
                    .first::<Uuid>(&mut *connection)
                    .await
                    .optional()?;
                match (enabled, existing) {
                    (true, None) => {
                        diesel::insert_into(role_permissions::table)
                            .values((
                                role_permissions::role_id.eq(role_type.id()),
                                role_permissions::permission_id.eq(permission_id),
                            ))
                            .execute(&mut *connection)
                            .await?;
                    }
                    (false, Some(role_permission_id)) => {
                        diesel::delete(
                            role_permissions::table.filter(
                                role_permissions::role_permission_id.eq(role_permission_id),
                            ),
                        )
                        .execute(&mut *connection)
                        .await?;
                    }
                    (true, Some(_)) | (false, None) => {
                        return Err(AuthorizationError::NoChange);
                    }
                }

                let (kind, old_value, new_value) = if enabled {
                    (
                        AuthorizationAuditKind::RolePermissionGranted,
                        "disabled",
                        "enabled",
                    )
                } else {
                    (
                        AuthorizationAuditKind::RolePermissionRevoked,
                        "enabled",
                        "disabled",
                    )
                };
                let audit_event_id = insert_permission_audit(
                    connection,
                    &actor,
                    role_type,
                    permission_id,
                    &permission_name,
                    kind,
                    old_value,
                    new_value,
                    reason,
                    request_id,
                )
                .await?;
                Ok(RolePermissionReceipt {
                    audit_event_id,
                    role_type,
                    permission_id,
                    permission_name,
                    enabled,
                })
            })
            .await
    }
}
struct LockedRoleTarget {
    role_type: RoleType,
}
async fn lock_role_target(
    connection: &mut diesel_async::AsyncPgConnection,
    user_id: Uuid,
) -> Result<LockedRoleTarget, AuthorizationError> {
    let user = users::table
        .filter(users::user_id.eq(user_id))
        .select((
            users::user_is_system_actor,
            users::user_deleted_at,
            users::user_hard_purged_at,
        ))
        .for_update()
        .first::<(
            bool,
            Option<chrono::DateTime<chrono::Utc>>,
            Option<chrono::DateTime<chrono::Utc>>,
        )>(&mut *connection)
        .await
        .optional()?
        .ok_or(AuthorizationError::AccountNotFound)?;
    if user.0 {
        return Err(AuthorizationError::SystemActorProtected);
    }
    if user.1.is_some() || user.2.is_some() {
        return Err(AuthorizationError::AccountNotFound);
    }
    let role_id = user_roles::table
        .filter(user_roles::user_id.eq(user_id))
        .select(user_roles::role_id)
        .for_update()
        .first::<Uuid>(&mut *connection)
        .await
        .optional()?
        .ok_or(AuthorizationError::AccountNotFound)?;
    let role_type =
        RoleType::from_uuid(role_id).ok_or(AuthorizationError::InvalidRoleId(role_id))?;
    Ok(LockedRoleTarget { role_type })
}

async fn lock_known_role(
    connection: &mut diesel_async::AsyncPgConnection,
    role_type: RoleType,
) -> Result<(), AuthorizationError> {
    let role_name = roles::table
        .filter(roles::role_id.eq(role_type.id()))
        .select(roles::role_name)
        .for_update()
        .first::<String>(&mut *connection)
        .await
        .optional()?;
    match role_name.as_deref() {
        Some(name) if name == role_type.name() => Ok(()),
        Some(_) | None => Err(AuthorizationError::InvalidRoleId(role_type.id())),
    }
}

async fn lock_permission(
    connection: &mut diesel_async::AsyncPgConnection,
    permission_id: Uuid,
) -> Result<PermissionName, AuthorizationError> {
    let permission_name = permissions::table
        .filter(permissions::permission_id.eq(permission_id))
        .select(permissions::permission_name)
        .for_update()
        .first::<String>(&mut *connection)
        .await
        .optional()?
        .ok_or(AuthorizationError::PermissionNotFound)?;
    PermissionName::try_new(permission_name).map_err(|_| AuthorizationError::InvalidPermissionName)
}

async fn insert_audit_event(
    connection: &mut diesel_async::AsyncPgConnection,
    event: NewAuthorizationAuditEventRecord<'_>,
) -> Result<Uuid, AuthorizationError> {
    diesel::insert_into(authorization_audit_events::table)
        .values(event)
        .returning(authorization_audit_events::authorization_audit_event_id)
        .get_result::<Uuid>(&mut *connection)
        .await
        .map_err(AuthorizationError::Query)
}

#[allow(clippy::too_many_arguments)]
async fn insert_permission_audit(
    connection: &mut diesel_async::AsyncPgConnection,
    actor: &LockedYounghyun,
    role_type: RoleType,
    permission_id: Uuid,
    permission_name: &PermissionName,
    kind: AuthorizationAuditKind,
    old_value: &'static str,
    new_value: &'static str,
    reason: &AuthorizationReason,
    request_id: Option<Uuid>,
) -> Result<Uuid, AuthorizationError> {
    insert_audit_event(
        connection,
        NewAuthorizationAuditEventRecord {
            authorization_audit_event_actor_user_id: actor.user_id,
            authorization_audit_event_kind: kind.into(),
            authorization_audit_event_target_user_id: None,
            authorization_audit_event_role_id: role_type.id(),
            authorization_audit_event_role_name: role_type.name(),
            authorization_audit_event_permission_id: Some(permission_id),
            authorization_audit_event_permission_name: Some(permission_name.as_ref()),
            authorization_audit_event_old_value: old_value,
            authorization_audit_event_new_value: new_value,
            authorization_audit_event_reason: reason.as_ref(),
            authorization_audit_event_request_id: request_id,
        },
    )
    .await
}
