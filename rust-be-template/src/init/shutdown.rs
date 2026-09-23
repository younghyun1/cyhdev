//! Signal-driven shutdown: drain the listeners, then run a bounded list of cleanup hooks.
//!
//! Hooks run in registration order after in-flight requests have drained, so work those
//! requests buffered (visitor rows, view counts) is included. Each hook gets at most
//! [`PER_HOOK_LIMIT`] and the list shares [`HOOK_BUDGET`]; a hook that overruns is
//! abandoned and the next one still runs while budget remains. Anything left unflushed
//! is lost exactly as it would be on a crash, which the periodic flush jobs already
//! bound to about one minute of data.

use std::{future::Future, pin::Pin, sync::Arc, time::Duration};

use tokio::time::Instant;
use tracing::{error, info, warn};

use crate::init::state::ServerState;

/// Time allowed for in-flight HTTP requests after the listeners stop accepting.
pub const DRAIN_DEADLINE: Duration = Duration::from_secs(10);
/// Total time for every shutdown hook together.
pub const HOOK_BUDGET: Duration = Duration::from_secs(10);
/// Longest any single hook may run.
pub const PER_HOOK_LIMIT: Duration = Duration::from_secs(5);

type HookFuture = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

struct ShutdownHook {
    name: &'static str,
    run: Box<dyn FnOnce() -> HookFuture + Send>,
}

/// Result of one hook, reported for logs and tests.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HookOutcome {
    Completed,
    Failed,
    TimedOut,
    /// The shared budget ran out before the hook started.
    Skipped,
}

/// An ordered list of named async cleanup steps with a shared time budget.
pub struct ShutdownHooks {
    hooks: Vec<ShutdownHook>,
    total_budget: Duration,
    per_hook_limit: Duration,
}

impl ShutdownHooks {
    pub fn new(total_budget: Duration, per_hook_limit: Duration) -> Self {
        Self {
            hooks: Vec::new(),
            total_budget,
            per_hook_limit,
        }
    }

    /// Appends a hook; `run` is called once, only if budget remains when its turn comes.
    pub fn with_hook<F, Fut>(mut self, name: &'static str, run: F) -> Self
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
    {
        self.hooks.push(ShutdownHook {
            name,
            run: Box::new(move || Box::pin(run())),
        });
        self
    }

    /// Runs every hook in order and returns each outcome with its name.
    pub async fn run(self) -> Vec<(&'static str, HookOutcome)> {
        let deadline = Instant::now() + self.total_budget;
        let mut outcomes = Vec::with_capacity(self.hooks.len());
        for hook in self.hooks {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                warn!(hook = hook.name, "Shutdown hook skipped; budget exhausted");
                outcomes.push((hook.name, HookOutcome::Skipped));
                continue;
            }
            let started = Instant::now();
            let limit = remaining.min(self.per_hook_limit);
            let outcome = match tokio::time::timeout(limit, (hook.run)()).await {
                Ok(Ok(())) => HookOutcome::Completed,
                Ok(Err(error)) => {
                    error!(hook = hook.name, error = %error, "Shutdown hook failed");
                    HookOutcome::Failed
                }
                Err(_) => {
                    error!(
                        hook = hook.name,
                        limit_ms = limit.as_millis(),
                        "Shutdown hook timed out"
                    );
                    HookOutcome::TimedOut
                }
            };
            info!(
                hook = hook.name,
                outcome = ?outcome,
                elapsed_ms = started.elapsed().as_millis(),
                "Shutdown hook finished"
            );
            outcomes.push((hook.name, outcome));
        }
        outcomes
    }
}

/// Hooks for buffered state that would otherwise be lost on a normal restart.
///
/// Add further steps here in the order they must run, for example closing open
/// calls before flushing counters that those calls update.
pub fn server_shutdown_hooks(state: &Arc<ServerState>) -> ShutdownHooks {
    let live_chat = state.live_chat_service();
    let visitors = state.visitor_service();
    let photographs = state.photography_service();
    let blog = state.blog_service();
    ShutdownHooks::new(HOOK_BUDGET, PER_HOOK_LIMIT)
        .with_hook("close_open_calls", move || async move {
            live_chat
                .close_open_calls()
                .await
                .map(|_| ())
                .map_err(|error| anyhow::anyhow!("{error}"))
        })
        .with_hook("flush_visitor_logs", move || async move {
            visitors.flush().await.map(|_| ())
        })
        .with_hook("flush_photograph_views", move || async move {
            photographs
                .flush_views()
                .await
                .map(|_| ())
                .map_err(|error| anyhow::anyhow!("{error}"))
        })
        .with_hook("flush_blog_views", move || async move {
            blog.flush_views()
                .await
                .map(|_| ())
                .map_err(|error| anyhow::anyhow!("{error}"))
        })
}

/// Resolves on SIGTERM or Ctrl-C and names the signal received.
///
/// If the SIGTERM handler cannot be installed, Ctrl-C still works; if neither can,
/// this never resolves and the process keeps serving, matching the previous
/// behaviour rather than exiting at startup.
pub async fn wait_for_signal() -> &'static str {
    let ctrl_c = async {
        match tokio::signal::ctrl_c().await {
            Ok(()) => "SIGINT",
            Err(error) => {
                error!(error = %error, "Ctrl-C handler unavailable");
                std::future::pending::<&'static str>().await
            }
        }
    };
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
                "SIGTERM"
            }
            Err(error) => {
                error!(error = %error, "SIGTERM handler unavailable");
                std::future::pending::<&'static str>().await
            }
        }
    };
    tokio::select! {
        signal = ctrl_c => signal,
        signal = terminate => signal,
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    use super::{HookOutcome, ShutdownHooks};

    #[tokio::test]
    async fn hooks_run_in_order_and_one_failure_does_not_stop_the_rest() {
        let order = Arc::new(Mutex::new(Vec::new()));
        let (first, second) = (Arc::clone(&order), Arc::clone(&order));
        let outcomes = ShutdownHooks::new(Duration::from_secs(1), Duration::from_secs(1))
            .with_hook("first", move || async move {
                if let Ok(mut order) = first.lock() {
                    order.push("first");
                }
                Err(anyhow::anyhow!("flush failed"))
            })
            .with_hook("second", move || async move {
                if let Ok(mut order) = second.lock() {
                    order.push("second");
                }
                Ok(())
            })
            .run()
            .await;
        assert_eq!(
            outcomes,
            vec![
                ("first", HookOutcome::Failed),
                ("second", HookOutcome::Completed)
            ]
        );
        assert_eq!(
            order.lock().map(|order| order.clone()).ok(),
            Some(vec!["first", "second"])
        );
    }

    #[tokio::test]
    async fn slow_hooks_are_cut_off_and_exhausted_budget_skips_the_rest() {
        let outcomes = ShutdownHooks::new(Duration::from_millis(150), Duration::from_millis(100))
            .with_hook("stuck", || async {
                tokio::time::sleep(Duration::from_secs(30)).await;
                Ok(())
            })
            .with_hook("also_stuck", || async {
                tokio::time::sleep(Duration::from_secs(30)).await;
                Ok(())
            })
            .with_hook("never_started", || async { Ok(()) })
            .run()
            .await;
        assert_eq!(
            outcomes,
            vec![
                ("stuck", HookOutcome::TimedOut),
                ("also_stuck", HookOutcome::TimedOut),
                ("never_started", HookOutcome::Skipped)
            ]
        );
    }
}
