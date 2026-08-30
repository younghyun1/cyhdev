use std::sync::Arc;
use std::time::Duration;

use tracing::{error, info};

use crate::{
    init::state::ServerState,
    jobs::{
        auth::{
            invalidate_sessions::invalidate_sessions,
            update_system_stats::update_system_stats,
        },
        job_funcs::{
            every_day::schedule_task_every_day_at, every_hour::schedule_task_every_hour_at,
            every_minute::schedule_task_every_minute_at,
            every_second::schedule_task_every_second_at,
        },
        maintenance::{
            compress_logs::compress_old_logs, flush_photograph_views::flush_photograph_views,
            flush_visitor_logs::flush_visitor_logs, prune_live_chat::prune_live_chat_state,
            prune_auth_abuse::prune_auth_abuse,
            prune_photograph_batches::prune_photograph_batches,
            retry_media_cleanup::retry_media_object_cleanup,
            send_retention_notifications::send_retention_notifications,
        },
    },
};

/// Spawn a supervised scheduler loop.
///
/// Each periodic scheduler is itself an infinite loop, so the spawned task IS the
/// job. If it panics (or returns unexpectedly), the job would silently stop forever.
/// `supervise` owns the `JoinHandle`, logs the failure, and restarts the scheduler
/// after an exponential backoff (capped at 60s).
///
/// `make` is a factory that rebuilds the scheduler future on each attempt. The task
/// closures captured by the schedulers are not `Clone`, so the factory re-clones the
/// shared `Arc<ServerState>` and reconstructs the closure per restart.
fn supervise<F, Fut>(name: &'static str, make: F)
where
    F: Fn() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = anyhow::Result<()>> + Send + 'static,
{
    tokio::spawn(async move {
        let mut backoff = Duration::from_secs(1);
        loop {
            // Run the scheduler in its own task so a panic surfaces as a join error
            // instead of unwinding the supervisor.
            match tokio::spawn(make()).await {
                Ok(Ok(())) => {
                    error!(
                        task = name,
                        "scheduler loop returned unexpectedly; restarting"
                    )
                }
                Ok(Err(e)) => {
                    error!(task = name, error = ?e, "scheduler exited with error; restarting")
                }
                Err(join_err) => {
                    error!(task = name, error = %join_err, "scheduler task panicked/aborted; restarting")
                }
            }
            tokio::time::sleep(backoff).await;
            backoff = (backoff * 2).min(Duration::from_secs(60));
        }
    });
}

pub async fn task_init(state: Arc<ServerState>) -> anyhow::Result<()> {
    info!("Task scheduler running...");

    // Startup sweep: clear orphaned batch temp dirs from a previous process run.
    // The in-memory tracker is empty at startup, so the whole staging root is stale.
    tokio::spawn(async {
        let root = crate::util::image::batch_pipeline::batch_root_dir();
        if let Err(e) = tokio::fs::remove_dir_all(&root).await
            && e.kind() != std::io::ErrorKind::NotFound
        {
            tracing::warn!(error = %e, path = %root.display(), "Failed startup sweep of batch temp dir");
        }
    });

    {
        let state = Arc::clone(&state);
        supervise("INVALIDATE_EXPIRED_SESSIONS", move || {
            let state = Arc::clone(&state);
            schedule_task_every_hour_at(
                state,
                move |coroutine_state: Arc<ServerState>| async move {
                    invalidate_sessions(coroutine_state).await
                },
                String::from("INVALIDATE_EXPIRED_SESSIONS"),
                30, // minutes
                00, // seconds
            )
        });
    }

    {
        let state = Arc::clone(&state);
        supervise("UPDATE_SYSTEM_STATS", move || {
            let state = Arc::clone(&state);
            schedule_task_every_second_at(
                state,
                move |coroutine_state: Arc<ServerState>| async move {
                    update_system_stats(coroutine_state).await
                },
                String::from("UPDATE_SYSTEM_STATS"),
                0,
                0,
            )
        });
    }

    {
        let state = Arc::clone(&state);
        supervise("COMPRESS_OLD_LOGS", move || {
            let state = Arc::clone(&state);
            schedule_task_every_day_at::<_, _>(
                state,
                move |coroutine_state: Arc<ServerState>| async move {
                    compress_old_logs(coroutine_state).await
                },
                String::from("COMPRESS_OLD_LOGS"),
                6,
                30,
                00,
            )
        });
    }

    {
        let state = Arc::clone(&state);
        supervise("FLUSH_VISITOR_LOGS", move || {
            let state = Arc::clone(&state);
            schedule_task_every_minute_at(
                state,
                move |coroutine_state: Arc<ServerState>| async move {
                    flush_visitor_logs(coroutine_state).await
                },
                String::from("FLUSH_VISITOR_LOGS"),
                0,
                0,
            )
        });
    }

    {
        let state = Arc::clone(&state);
        supervise("PRUNE_AUTH_ABUSE", move || {
            let state = Arc::clone(&state);
            schedule_task_every_minute_at(
                state,
                move |coroutine_state: Arc<ServerState>| async move {
                    prune_auth_abuse(coroutine_state).await
                },
                String::from("PRUNE_AUTH_ABUSE"),
                10,
                0,
            )
        });
    }

    {
        let state = Arc::clone(&state);
        supervise("PRUNE_LIVE_CHAT_STATE", move || {
            let state = Arc::clone(&state);
            schedule_task_every_minute_at(
                state,
                move |coroutine_state: Arc<ServerState>| async move {
                    prune_live_chat_state(coroutine_state).await
                },
                String::from("PRUNE_LIVE_CHAT_STATE"),
                30,
                0,
            )
        });
    }

    {
        let state = Arc::clone(&state);
        supervise("FLUSH_PHOTOGRAPH_VIEWS", move || {
            let state = Arc::clone(&state);
            schedule_task_every_minute_at(
                state,
                move |coroutine_state: Arc<ServerState>| async move {
                    flush_photograph_views(coroutine_state).await
                },
                String::from("FLUSH_PHOTOGRAPH_VIEWS"),
                15,
                0,
            )
        });
    }

    {
        let state = Arc::clone(&state);
        supervise("PRUNE_PHOTOGRAPH_BATCHES", move || {
            let state = Arc::clone(&state);
            schedule_task_every_minute_at(
                state,
                move |coroutine_state: Arc<ServerState>| async move {
                    prune_photograph_batches(coroutine_state).await
                },
                String::from("PRUNE_PHOTOGRAPH_BATCHES"),
                45,
                0,
            )
        });
    }

    {
        let state = Arc::clone(&state);
        supervise("RETRY_MEDIA_OBJECT_CLEANUP", move || {
            let state = Arc::clone(&state);
            schedule_task_every_minute_at(
                state,
                move |coroutine_state: Arc<ServerState>| async move {
                    retry_media_object_cleanup(coroutine_state).await
                },
                String::from("RETRY_MEDIA_OBJECT_CLEANUP"),
                20,
                0,
            )
        });
    }

    {
        let state = Arc::clone(&state);
        supervise("SEND_RETENTION_NOTIFICATIONS", move || {
            let state = Arc::clone(&state);
            schedule_task_every_minute_at(
                state,
                move |coroutine_state: Arc<ServerState>| async move {
                    send_retention_notifications(coroutine_state).await
                },
                String::from("SEND_RETENTION_NOTIFICATIONS"),
                40,
                0,
            )
        });
    }

    Ok(())
}
