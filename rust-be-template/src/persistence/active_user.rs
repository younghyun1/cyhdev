//! Transaction-local account authority for authenticated content writes.

use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    features::accounts::domain::role::RoleType,
    schema::{user_roles, users},
};

#[derive(Debug, thiserror::Error)]
pub enum ActiveUserWriteError {
    #[error("authenticated account is no longer active")]
    Inactive,
    #[error("authenticated account cannot mutate this content")]
    Denied,
    #[error("content target does not exist")]
    TargetNotFound,
    #[error("content write failed: {0}")]
    Database(#[from] diesel::result::Error),
}

/// Lock the authoritative account row before an authenticated content mutation.
///
/// Callers take this lock before any content row, so every content write uses
/// the same user-then-content order. `FOR SHARE` suffices because the check
/// only asserts the actor is active: lifecycle deletion takes `FOR UPDATE` on
/// the same row, which conflicts with a share lock. Keeping this guard and the
/// write in one transaction means the write either commits before deletion, or
/// waits for it, re-evaluates `user_deleted_at` against the committed row, and
/// fails without mutating retained content. Share locks do not conflict with
/// each other, so one account's concurrent writes no longer queue on its row.
pub async fn lock_active_user(
    conn: &mut AsyncPgConnection,
    user_id: Uuid,
) -> Result<(), ActiveUserWriteError> {
    let active_user_id = users::table
        .filter(users::user_id.eq(user_id))
        .filter(users::user_deleted_at.is_null())
        .select(users::user_id)
        .for_share()
        .first::<Uuid>(conn)
        .await
        .optional()?;
    match active_user_id {
        Some(_) => Ok(()),
        None => Err(ActiveUserWriteError::Inactive),
    }
}

/// Lock and recheck both active account state and current superuser authority.
///
/// Role assignment updates the `user_roles` row, which conflicts with this
/// share lock, so the role cannot change before the caller's write commits.
pub async fn lock_active_superuser(
    conn: &mut AsyncPgConnection,
    user_id: Uuid,
) -> Result<(), ActiveUserWriteError> {
    lock_active_user(conn, user_id).await?;
    let role_id = user_roles::table
        .filter(user_roles::user_id.eq(user_id))
        .select(user_roles::role_id)
        .for_share()
        .first::<Uuid>(conn)
        .await
        .optional()?;
    match role_id.and_then(RoleType::from_uuid) {
        Some(role) if role.is_superuser() => Ok(()),
        _ => Err(ActiveUserWriteError::Denied),
    }
}
