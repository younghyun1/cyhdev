//! Database-current Younghyun authority checks for privileged account work.

use diesel::{ExpressionMethods, QueryDsl, dsl::exists};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    features::accounts::{authorization_error::AuthorizationError, domain::role::RoleType},
    schema::{user_roles, users},
};

#[derive(Clone, Debug)]
pub(super) struct LockedYounghyun {
    pub(super) user_id: Uuid,
}

pub(super) async fn ensure_current_younghyun(
    connection: &mut diesel_async::AsyncPgConnection,
    actor_user_id: Uuid,
) -> Result<(), AuthorizationError> {
    let current = user_roles::table
        .inner_join(users::table)
        .filter(user_roles::user_id.eq(actor_user_id))
        .filter(user_roles::role_id.eq(RoleType::Younghyun.id()))
        .filter(users::user_deleted_at.is_null())
        .filter(users::user_hard_purged_at.is_null());
    let allowed = diesel::select(exists(current))
        .get_result::<bool>(&mut *connection)
        .await?;
    if allowed {
        Ok(())
    } else {
        Err(AuthorizationError::Unauthorized)
    }
}

/// Locks every active Younghyun assignment in stable order before a change.
/// Competing demotions therefore serialize and re-evaluate the same authority set.
pub(super) async fn lock_active_younghyun_authority(
    connection: &mut diesel_async::AsyncPgConnection,
    actor_user_id: Uuid,
) -> Result<(LockedYounghyun, usize), AuthorizationError> {
    let owners = user_roles::table
        .inner_join(users::table)
        .filter(user_roles::role_id.eq(RoleType::Younghyun.id()))
        .filter(users::user_deleted_at.is_null())
        .filter(users::user_hard_purged_at.is_null())
        .order(user_roles::user_id.asc())
        .select(user_roles::user_id)
        .for_update()
        .load::<Uuid>(&mut *connection)
        .await?;
    let owner_count = owners.len();
    match owners.into_iter().find(|user_id| *user_id == actor_user_id) {
        Some(user_id) => Ok((LockedYounghyun { user_id }, owner_count)),
        None => Err(AuthorizationError::Unauthorized),
    }
}
