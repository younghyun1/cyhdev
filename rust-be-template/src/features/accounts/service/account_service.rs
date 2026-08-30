//! Explicit dependencies for account use cases.

use std::sync::Arc;

use lettre::{AsyncSmtpTransport, Tokio1Executor};

use crate::{
    features::{accounts::{
        repository::account_repository::AccountRepository,
        service::session_service::SessionService,
    }, live_chat::service::lifecycle::LiveChatAccountLifecyclePort},
    util::media::object_store::MediaObjectStore,
};

pub const MAX_PASSWORD_JOBS: usize = 4;
pub const MAX_EMAIL_JOBS: usize = 16;

pub struct AccountServiceDependencies {
    pub repository: Arc<AccountRepository>,
    pub sessions: Arc<SessionService>,
    pub live_chat_lifecycle: Arc<dyn LiveChatAccountLifecyclePort>,
    pub media_object_store: Arc<dyn MediaObjectStore>,
    pub media_region: Arc<str>,
    pub email_client: AsyncSmtpTransport<Tokio1Executor>,
    pub public_app_origin: Arc<str>,
    pub dummy_password_hash: String,
}

/// Coordinates account use cases across persistence, sessions, and email delivery.
pub struct AccountService {
    pub(super) repository: Arc<AccountRepository>,
    pub(super) sessions: Arc<SessionService>,
    pub(super) live_chat_lifecycle: Arc<dyn LiveChatAccountLifecyclePort>,
    pub(super) media_object_store: Arc<dyn MediaObjectStore>,
    pub(super) media_region: Arc<str>,
    pub(super) email_client: Arc<AsyncSmtpTransport<Tokio1Executor>>,
    pub(super) public_app_origin: Arc<str>,
    pub(super) dummy_password_hash: Arc<str>,
    pub(super) password_jobs: tokio::sync::Semaphore,
    pub(super) email_jobs: Arc<tokio::sync::Semaphore>,
    /// Excludes retained-email delivery while hard purge removes private identity.
    pub(super) retention_notification_delivery_gate: tokio::sync::RwLock<()>,
    /// Prevents one process from overlapping retention-notification batches.
    pub(super) retention_notification_run_gate: tokio::sync::Mutex<()>,
    /// Prevents a login from creating a stale session while an account mutation commits.
    pub(super) session_consistency: tokio::sync::RwLock<()>,
}

impl AccountService {
    pub fn new(dependencies: AccountServiceDependencies) -> Self {
        let AccountServiceDependencies {
            repository,
            sessions,
            live_chat_lifecycle,
            media_object_store,
            media_region,
            email_client,
            public_app_origin,
            dummy_password_hash,
        } = dependencies;
        Self {
            repository,
            sessions,
            live_chat_lifecycle,
            media_object_store,
            media_region,
            email_client: Arc::new(email_client),
            public_app_origin,
            dummy_password_hash: Arc::from(dummy_password_hash),
            password_jobs: tokio::sync::Semaphore::new(MAX_PASSWORD_JOBS),
            email_jobs: Arc::new(tokio::sync::Semaphore::new(MAX_EMAIL_JOBS)),
            retention_notification_delivery_gate: tokio::sync::RwLock::new(()),
            retention_notification_run_gate: tokio::sync::Mutex::new(()),
            session_consistency: tokio::sync::RwLock::new(()),
        }
    }

    pub(super) fn try_password_job(
        &self,
    ) -> Result<tokio::sync::SemaphorePermit<'_>, crate::features::accounts::error::AccountError>
    {
        self.password_jobs.try_acquire().map_err(|_| {
            crate::features::accounts::error::AccountError::PasswordWorkSaturated {
                max_jobs: MAX_PASSWORD_JOBS,
            }
        })
    }
}
