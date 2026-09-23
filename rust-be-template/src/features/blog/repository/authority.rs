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

/// Asserts the acting account is active and returns its current role.
///
/// Callers lock the actor before any content row so every blog write takes
/// the same user-then-content order. `FOR SHARE` is enough: soft deletion
/// takes `FOR UPDATE` on the user row and role assignment updates the
/// `user_roles` row, and both conflict with a share lock. Such a transaction
/// therefore waits for this write to commit, or this write waits for it and
/// then re-evaluates the filters against the committed row and fails. Share
/// locks do not conflict with each other, so one account's concurrent writes
/// no longer serialize on its user row.
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
        .for_share()
        .first::<Uuid>(&mut *connection)
        .await
        .optional()?;
    if active.is_none() {
        return Err(BlogError::Unauthorized);
    }
    let role_id = user_roles::table
        .filter(user_roles::user_id.eq(user_id))
        .select(user_roles::role_id)
        .for_share()
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

/// Checks ownership after the requester is already locked by the caller.
pub(super) fn require_owner_or_superuser(
    requester_id: Uuid,
    requester_role: RoleType,
    owner_id: Uuid,
) -> Result<(), BlogError> {
    if requester_id == owner_id || requester_role.is_superuser() {
        Ok(())
    } else {
        Err(BlogError::Forbidden)
    }
}

/// Rejects interaction with an unpublished post unless the actor manages the blog.
///
/// A draft is reported as absent so its existence is not disclosed.
pub(super) async fn require_visible_post(
    connection: &mut AsyncPgConnection,
    actor_id: Uuid,
    post_is_published: bool,
) -> Result<(), BlogError> {
    if post_is_published || has_current_blog_authority(connection, Some(actor_id)).await? {
        Ok(())
    } else {
        Err(BlogError::PostNotFound)
    }
}
