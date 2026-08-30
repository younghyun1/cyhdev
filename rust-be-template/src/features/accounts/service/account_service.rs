//! Explicit dependencies for account use cases.

use std::sync::Arc;

use lettre::{AsyncSmtpTransport, Tokio1Executor};

use crate::features::accounts::{
    repository::account_repository::AccountRepository, service::session_service::SessionService,
};

/// Coordinates account use cases across persistence, sessions, and email delivery.
pub struct AccountService {
    pub(super) repository: Arc<AccountRepository>,
    pub(super) sessions: Arc<SessionService>,
    pub(super) email_client: Arc<AsyncSmtpTransport<Tokio1Executor>>,
    /// Prevents a login from creating a stale session while an account mutation commits.
    pub(super) session_consistency: tokio::sync::RwLock<()>,
}

impl AccountService {
    pub fn new(
        repository: Arc<AccountRepository>,
        sessions: Arc<SessionService>,
        email_client: AsyncSmtpTransport<Tokio1Executor>,
    ) -> Self {
        Self {
            repository,
            sessions,
            email_client: Arc::new(email_client),
            session_consistency: tokio::sync::RwLock::new(()),
        }
    }
}
