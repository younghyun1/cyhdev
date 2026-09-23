//! Bounded Argon2 work with separate budgets for authentication and confirmations.

use std::sync::Arc;

use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use zeroize::Zeroizing;

use crate::{
    features::accounts::{error::AccountError, service::account_service::AccountService},
    util::crypto::{hash_pw::hash_pw_blocking, verify_pw::verify_pw_blocking},
};

/// Concurrent Argon2 jobs for login, signup, and password reset.
pub const MAX_PASSWORD_JOBS: usize = 4;
/// Concurrent Argon2 jobs for authenticated current-password confirmations.
///
/// Confirmations draw from their own pool so a signed-in account cannot keep login at 429 by
/// repeating profile, deletion, or OIDC confirmations.
pub const MAX_CONFIRMATION_PASSWORD_JOBS: usize = 2;

/// The Argon2 pool a use case draws from.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PasswordBudget {
    Authentication,
    Confirmation,
}

/// Fixed pools behind [`PasswordBudget`].
pub(super) struct PasswordJobs {
    authentication: Arc<Semaphore>,
    confirmation: Arc<Semaphore>,
}

impl PasswordJobs {
    pub(super) fn new() -> Self {
        Self {
            authentication: Arc::new(Semaphore::new(MAX_PASSWORD_JOBS)),
            confirmation: Arc::new(Semaphore::new(MAX_CONFIRMATION_PASSWORD_JOBS)),
        }
    }

    /// Takes a slot without waiting; saturation rejects before any blocking task exists.
    fn try_acquire(&self, budget: PasswordBudget) -> Result<OwnedSemaphorePermit, AccountError> {
        let (pool, max_jobs) = match budget {
            PasswordBudget::Authentication => (&self.authentication, MAX_PASSWORD_JOBS),
            PasswordBudget::Confirmation => (&self.confirmation, MAX_CONFIRMATION_PASSWORD_JOBS),
        };
        Arc::clone(pool)
            .try_acquire_owned()
            .map_err(|_| AccountError::PasswordWorkSaturated { max_jobs })
    }
}

impl AccountService {
    /// Verifies a password on the blocking pool while holding a slot of `budget`.
    pub(super) async fn verify_password(
        &self,
        budget: PasswordBudget,
        password: &str,
        expected_hash: &str,
    ) -> Result<bool, AccountError> {
        let permit = self.password_jobs.try_acquire(budget)?;
        let password = Zeroizing::new(password.to_owned());
        let expected_hash = Zeroizing::new(expected_hash.to_owned());
        // The owned permit moves into the closure, so cancelling the request future cannot
        // release the slot while Argon2 keeps running on the blocking pool.
        let result = tokio::task::spawn_blocking(move || {
            let _permit = permit;
            verify_pw_blocking(password.as_bytes(), &expected_hash)
        })
        .await;
        match result {
            Ok(verified) => verified.map_err(AccountError::PasswordVerification),
            Err(join_error) => Err(AccountError::PasswordVerification(anyhow::Error::new(
                join_error,
            ))),
        }
    }

    /// Hashes a password on the blocking pool while holding a slot of `budget`.
    pub(super) async fn hash_password(
        &self,
        budget: PasswordBudget,
        password: Zeroizing<String>,
    ) -> Result<String, AccountError> {
        let permit = self.password_jobs.try_acquire(budget)?;
        let result = tokio::task::spawn_blocking(move || {
            let _permit = permit;
            hash_pw_blocking(password.as_bytes())
        })
        .await;
        match result {
            Ok(hashed) => hashed.map_err(AccountError::PasswordHash),
            Err(join_error) => Err(AccountError::PasswordHash(anyhow::Error::new(join_error))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confirmation_saturation_leaves_authentication_slots_free() {
        let jobs = PasswordJobs::new();
        let confirmations = (0..MAX_CONFIRMATION_PASSWORD_JOBS)
            .map(|_| jobs.try_acquire(PasswordBudget::Confirmation))
            .collect::<Result<Vec<_>, _>>();
        assert!(confirmations.is_ok());
        assert!(matches!(
            jobs.try_acquire(PasswordBudget::Confirmation),
            Err(AccountError::PasswordWorkSaturated {
                max_jobs: MAX_CONFIRMATION_PASSWORD_JOBS
            })
        ));
        assert!(jobs.try_acquire(PasswordBudget::Authentication).is_ok());
    }

    #[tokio::test]
    async fn permit_outlives_a_cancelled_waiter_until_the_blocking_job_ends() {
        let jobs = PasswordJobs::new();
        let permit = jobs.try_acquire(PasswordBudget::Confirmation);
        assert!(permit.is_ok());
        let (release_tx, release_rx) = std::sync::mpsc::channel::<()>();
        let blocking = tokio::task::spawn_blocking(move || {
            let _permit = permit;
            let _ = release_rx.recv();
        });
        // Dropping the awaiting future stands in for a cancelled request.
        drop(tokio::time::timeout(std::time::Duration::from_millis(10), blocking).await);
        let available = jobs.confirmation.available_permits();
        let _ = release_tx.send(());
        assert_eq!(available, MAX_CONFIRMATION_PASSWORD_JOBS - 1);
    }
}
