//! Post-commit session refresh, fail-closed revocation, and cancellation-safe execution.

use std::future::Future;

use tracing::{error, trace};
use uuid::Uuid;

use crate::features::accounts::{
    domain::role::RoleType, error::AccountError, service::account_service::AccountService,
};

/// Runs a commit together with its post-commit session work on a spawned task.
///
/// A request future is dropped when its client disconnects. Awaiting the spawned handle
/// means that drop cancels only the wait: the task still finishes both the commit and the
/// session revocation, refresh, or live-chat anonymization that must follow it, so a
/// committed password reset or deletion can never leave stale sessions alive.
pub(super) async fn run_to_completion<T, E, F>(work: F) -> Result<T, E>
where
    T: Send + 'static,
    E: From<AccountError> + Send + 'static,
    F: Future<Output = Result<T, E>> + Send + 'static,
{
    match tokio::spawn(work).await {
        Ok(result) => result,
        Err(join_error) => {
            error!(
                event = "account_mutation_task_failed",
                error = %join_error,
                "Detached account mutation task failed"
            );
            Err(AccountError::BackgroundTask(join_error).into())
        }
    }
}

impl AccountService {
    pub(super) async fn refresh_sessions_after_commit(
        &self,
        user_id: Uuid,
        mutation: &'static str,
    ) {
        match self.session_snapshot(user_id).await {
            Ok((account, role_type)) => {
                let refreshed = self
                    .sessions
                    .refresh_for_user(user_id, &account, role_type)
                    .await;
                trace!(%user_id, mutation, refreshed, "Refreshed sessions after account mutation");
            }
            Err(error) => {
                let revoked = self.sessions.remove_for_user(user_id).await;
                error!(
                    %user_id,
                    mutation,
                    revoked,
                    retryable = error.is_retryable(),
                    error = %error,
                    "Revoked sessions after post-commit refresh failed"
                );
            }
        }
    }

    async fn session_snapshot(
        &self,
        user_id: Uuid,
    ) -> Result<
        (
            crate::features::accounts::domain::account::SessionAccount,
            RoleType,
        ),
        AccountError,
    > {
        let account = match self.repository.session_account(user_id).await? {
            Some(account) => account,
            None => return Err(AccountError::AccountNotFound),
        };
        let role_type = self
            .repository
            .role_for_user_or_insert_default(user_id, RoleType::User)
            .await?;
        Ok((account, role_type))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    use super::run_to_completion;
    use crate::features::accounts::error::AccountError;

    #[tokio::test]
    async fn dropping_the_waiter_does_not_cancel_the_follow_up() {
        let (commit_tx, commit_rx) = tokio::sync::oneshot::channel::<()>();
        let (done_tx, done_rx) = tokio::sync::oneshot::channel::<()>();
        let follow_up_ran = Arc::new(AtomicBool::new(false));
        let observed = Arc::clone(&follow_up_ran);
        let waiter = tokio::spawn(run_to_completion(async move {
            let _ = commit_rx.await;
            observed.store(true, Ordering::SeqCst);
            let _ = done_tx.send(());
            Ok::<(), AccountError>(())
        }));
        tokio::task::yield_now().await;
        // Aborting the waiter stands in for a disconnected client dropping its request.
        waiter.abort();
        let _ = waiter.await;
        let _ = commit_tx.send(());
        assert!(done_rx.await.is_ok());
        assert!(follow_up_ran.load(Ordering::SeqCst));
    }
}
