//! Short-lived lease that serializes forum writes with account authority changes.

use uuid::Uuid;

use crate::features::accounts::{
    authorization_error::AuthorizationError, domain::forum_authority::ForumActorAuthority,
    service::account_service::AccountService,
};

pub struct ForumAuthorityLease<'a> {
    _guard: tokio::sync::RwLockReadGuard<'a, ()>,
    authority: ForumActorAuthority,
}

impl ForumAuthorityLease<'_> {
    pub fn authority(&self) -> ForumActorAuthority {
        self.authority
    }
}

impl AccountService {
    pub async fn acquire_forum_authority(
        &self,
        user_id: Uuid,
    ) -> Result<ForumAuthorityLease<'_>, AuthorizationError> {
        let guard = self.session_consistency.read().await;
        let authority = self.repository.forum_actor_authority(user_id).await?;
        Ok(ForumAuthorityLease {
            _guard: guard,
            authority,
        })
    }
}
