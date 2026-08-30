//! Account authority exposed to short forum use cases.

use uuid::Uuid;

#[derive(Clone, Copy, Debug)]
pub struct ForumActorAuthority {
    pub user_id: Uuid,
    pub can_moderate: bool,
}
