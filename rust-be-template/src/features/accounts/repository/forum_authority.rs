//! Database-current forum posting and moderation authority.

use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, dsl::exists};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    features::accounts::{
        authorization_error::AuthorizationError, domain::forum_authority::ForumActorAuthority,
        repository::account_repository::AccountRepository,
    },
    schema::{permissions, role_permissions, user_roles, users},
};

pub const FORUM_MODERATE_PERMISSION: &str = "forum.moderate";

impl AccountRepository {
    pub async fn forum_actor_authority(
        &self,
        user_id: Uuid,
    ) -> Result<ForumActorAuthority, AuthorizationError> {
        let mut connection = self.connection().await?;
        let active = exists(
            users::table
                .filter(users::user_id.eq(user_id))
                .filter(users::user_deleted_at.is_null())
                .filter(users::user_hard_purged_at.is_null())
                .filter(users::user_is_email_verified.eq(true)),
        );
        let permission = user_roles::table
            .inner_join(
                role_permissions::table.on(role_permissions::role_id.eq(user_roles::role_id)),
            )
            .inner_join(
                permissions::table
                    .on(permissions::permission_id.eq(role_permissions::permission_id)),
            )
            .filter(user_roles::user_id.eq(user_id))
            .filter(permissions::permission_name.eq(FORUM_MODERATE_PERMISSION));
        // Both facts come from one statement and therefore one snapshot. The
        // permission probe also runs for an inactive account; its answer is
        // discarded with the error.
        let (active, can_moderate) = diesel::select((active, exists(permission)))
            .get_result::<(bool, bool)>(&mut connection)
            .await?;
        if !active {
            return Err(AuthorizationError::AccountNotFound);
        }
        Ok(ForumActorAuthority {
            user_id,
            can_moderate,
        })
    }
}
