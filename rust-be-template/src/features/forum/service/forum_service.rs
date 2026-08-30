//! Explicit forum dependencies.

use std::sync::Arc;

use crate::features::{
    accounts::service::account_service::AccountService,
    forum::repository::forum_repository::ForumRepository,
};

pub struct ForumService {
    pub(super) repository: Arc<ForumRepository>,
    pub(super) accounts: Arc<AccountService>,
    pub(super) write_limiter: super::write_limiter::ForumWriteLimiter,
}

impl ForumService {
    pub fn new(repository: Arc<ForumRepository>, accounts: Arc<AccountService>) -> Self {
        Self {
            repository,
            accounts,
            write_limiter: super::write_limiter::ForumWriteLimiter::new(),
        }
    }
}
