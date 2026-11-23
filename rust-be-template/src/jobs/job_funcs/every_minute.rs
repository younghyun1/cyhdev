use std::sync::Arc;

use anyhow::{Result, anyhow};
use jiff::{SignedDuration, Timestamp};
use tracing::{error, info};

use crate::init::state::ServerState;
use crate::util::time::duration_formatter::format_duration;
use crate::util::time::utc::{from_utc_datetime, to_utc_datetime};

/// Calculate the next UTC DateTime that lands on the current/next minute,
/// with a specific "seconds + milliseconds" offset from the start of that minute.
///
/// For example, second_offset=30 and millisecond_offset=500 would schedule
/// time XX:YY:30.500 of the next minute that is still >= now.
pub fn next_scheduled_mark(
    now: jiff::Timestamp,
    second_offset: u32,
    millisecond_offset: u32,
) -> Result<jiff::Timestamp> {
    // 1) Truncate to the current minute boundary (floor).
    let truncated_to_minute = match to_utc_datetime(now)
        .with()
        .second(0)
        .subsec_nanosecond(0)
        .build()
        .and_then(from_utc_datetime)
    {
        Ok(truncated_to_minute) => truncated_to_minute,
        Err(e) => {
            error!(now = %now, error = %e, "Could not truncate to minute");
            return Err(anyhow!("Could not truncate to minute"));
        }
    };

    // 2) Add the desired second + millisecond offset to get the target time this minute.
    let mut target_time = truncated_to_minute
        + SignedDuration::from_secs(i64::from(second_offset))
        + SignedDuration::from_millis(i64::from(millisecond_offset));

    // 3) If that target time is before 'now', then move it to the next minute by adding 60s.
    if target_time <= now {
        target_time += SignedDuration::from_secs(60);
    }

    Ok(target_time)
}

/// A helper that returns both (delay, next_mark).
/// It calculates how long until the next_scheduled_mark(...) from now.
pub fn next_scheduled_delay(
    _task_descriptor: &str,
    second_offset: u32,
    millisecond_offset: u32,
) -> Result<(tokio::time::Duration, jiff::Timestamp)> {
    let now = Timestamp::now();
    let next_mark = next_scheduled_mark(now, second_offset, millisecond_offset)?;

    // Convert that difference into std::time::Duration for tokio.
    let delay = std::time::Duration::try_from(next_mark.duration_since(now)).map_err(|e| {
        anyhow!(
            "Could not schedule job at next_scheduled_mark(). negative delay: {:?}",
            e
        )
    })?;

    Ok((delay, next_mark))
}

/// Schedules a task to run once per minute, but at a specific
/// second+millisecond offset (e.g., 30s + 500ms into every minute).
pub async fn schedule_task_every_minute_at<F, Fut>(
    state: Arc<ServerState>,
    task: F,
    task_descriptor: String,
    second_offset: u32,
    millisecond_offset: u32,
) -> Result<()>
where
    F: Fn(Arc<ServerState>) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    let mut initialized = false;
    let mut scheduled_run_time: Option<jiff::Timestamp> = None;

    loop {
        let (delay, next_mark) =
            match next_scheduled_delay(&task_descriptor, second_offset, millisecond_offset) {
                Ok((d, nm)) => (d, nm),
                Err(e) => {
                    error!(
                        task_name = %task_descriptor,
                        error = ?e,
                        "Could not calculate next scheduled time"
                    );
                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                    continue;
                }
            };

        if !initialized {
            info!(
                task_name = %task_descriptor,
                initial_run_time = %next_mark,
                delay = %format!("{:?}", delay),
                delay_human = %format_duration(delay),
                "Scheduled task initialized"
            );
            initialized = true;
        }

        let this_run_time = match scheduled_run_time {
            Some(rt) => rt,
            None => next_mark,
        };

        tokio::time::sleep(delay).await;

        let start = tokio::time::Instant::now();
        task(Arc::clone(&state)).await;
        let elapsed = start.elapsed();

        // Efficiently compute the next run time by simply adding one minute
        let next_run_time = this_run_time + SignedDuration::from_mins(1);
        let next_delay = match std::time::Duration::try_from(
            next_run_time.duration_since(Timestamp::now()),
        ) {
            Ok(next_delay) => next_delay,
            Err(e) => {
                error!(task_name = %task_descriptor, error = ?e, "Scheduled task next delay was negative");
                std::time::Duration::from_secs(60)
            }
        };

        info!(
            task_name = %task_descriptor,
            next_run_time = %next_run_time,
            duration = %format!("{:?}", elapsed),
            next_delay_human = %format_duration(next_delay),
            "Scheduled task ran"
        );

        scheduled_run_time = Some(next_run_time);
    }
}
