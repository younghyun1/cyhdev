use diesel::{ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    features::accounts::domain::role::RoleType,
    schema::{permissions, role_permissions, user_roles, users},
};

use super::super::error::BlogError;

pub(super) async fn has_current_blog_authority(
    connection: &mut AsyncPgConnection,
    user_id: Option<Uuid>,
) -> Result<bool, BlogError> {
    let Some(user_id) = user_id else {
        return Ok(false);
    };
    let current_authority = user_roles::table
        .inner_join(users::table)
        .inner_join(role_permissions::table.on(role_permissions::role_id.eq(user_roles::role_id)))
        .inner_join(
            permissions::table.on(permissions::permission_id.eq(role_permissions::permission_id)),
        )
        .filter(user_roles::user_id.eq(user_id))
        .filter(users::user_deleted_at.is_null())
        .filter(users::user_hard_purged_at.is_null())
        .filter(users::user_is_email_verified.eq(true))
        .filter(users::user_is_system_actor.eq(false))
        .filter(permissions::permission_name.eq("content.blog.manage"));
    diesel::select(diesel::dsl::exists(current_authority))
        .get_result::<bool>(&mut *connection)
        .await
        .map_err(BlogError::Database)
}

pub(super) async fn lock_active_user(
    connection: &mut AsyncPgConnection,
    user_id: Uuid,
) -> Result<RoleType, BlogError> {
    let active = users::table
        .filter(users::user_id.eq(user_id))
        .filter(users::user_deleted_at.is_null())
        .filter(users::user_hard_purged_at.is_null())
        .filter(users::user_is_email_verified.eq(true))
        .filter(users::user_is_system_actor.eq(false))
        .select(users::user_id)
        .for_update()
        .first::<Uuid>(&mut *connection)
        .await
        .optional()?;
    if active.is_none() {
        return Err(BlogError::Unauthorized);
    }
    let role_id = user_roles::table
        .filter(user_roles::user_id.eq(user_id))
        .select(user_roles::role_id)
        .for_update()
        .first::<Uuid>(&mut *connection)
        .await
        .optional()?;
    role_id
        .and_then(RoleType::from_uuid)
        .ok_or(BlogError::Unauthorized)
}

pub(super) async fn lock_active_superuser(
    connection: &mut AsyncPgConnection,
    user_id: Uuid,
) -> Result<(), BlogError> {
    let role = lock_active_user(connection, user_id).await?;
    if role.is_superuser() {
        Ok(())
    } else {
        Err(BlogError::Forbidden)
    }
}

pub(super) async fn require_owner_or_superuser(
    connection: &mut AsyncPgConnection,
    requester_id: Uuid,
    owner_id: Uuid,
) -> Result<(), BlogError> {
    let role = lock_active_user(connection, requester_id).await?;
    if requester_id == owner_id || role.is_superuser() {
        Ok(())
    } else {
        Err(BlogError::Forbidden)
    }
}
