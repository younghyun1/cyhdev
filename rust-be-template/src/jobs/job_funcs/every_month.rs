use std::sync::Arc;

use anyhow::{Result, anyhow};
use jiff::Timestamp;
use tracing::{error, info};

use crate::{
    init::state::ServerState,
    util::time::{
        duration_formatter::format_duration,
        utc::{from_utc_datetime, to_utc_datetime},
    },
};

fn validate_day_and_time(
    day_offset: u32,
    hour_offset: u32,
    minute_offset: u32,
    second_offset: u32,
) -> Result<()> {
    if !(1..=31).contains(&day_offset) {
        error!(day_offset, "Day offset is not in 1..=31");
        return Err(anyhow!("Invalid day offset for monthly schedule"));
    }
    if hour_offset > 23 || minute_offset > 59 || second_offset > 59 {
        error!(
            hour_offset,
            minute_offset, second_offset, "Bad schedule time: hour/minute/second out of range"
        );
        return Err(anyhow!("Invalid time offset for monthly schedule"));
    }
    Ok(())
}

fn next_month(year: i32, month: u32) -> (i32, u32) {
    if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    }
}

fn days_in_month(year: i32, month: u32) -> Result<u32> {
    let (Ok(civil_year), Ok(civil_month)) = (i16::try_from(year), i8::try_from(month)) else {
        error!(
            year,
            month, "Year/month out of range when building the monthly schedule"
        );
        return Err(anyhow!("Invalid year/month for monthly schedule"));
    };
    match jiff::civil::Date::new(civil_year, civil_month, 1) {
        Ok(start_of_month) => Ok(u32::from(start_of_month.days_in_month().unsigned_abs())),
        Err(e) => {
            error!(year, month, error = %e, "Invalid year/month when building the monthly schedule");
            Err(anyhow!("Invalid year/month for monthly schedule"))
        }
    }
}

fn build_month_mark(
    year: i32,
    month: u32,
    day_offset: u32,
    hour_offset: u32,
    minute_offset: u32,
    second_offset: u32,
) -> Result<jiff::Timestamp> {
    let days = days_in_month(year, month)?;
    let day = day_offset.min(days);
    let marker = match (
        i16::try_from(year),
        i8::try_from(month),
        i8::try_from(day),
        i8::try_from(hour_offset),
        i8::try_from(minute_offset),
        i8::try_from(second_offset),
    ) {
        (Ok(year), Ok(month), Ok(day), Ok(hour), Ok(minute), Ok(second)) => {
            jiff::civil::DateTime::new(year, month, day, hour, minute, second, 0)
                .and_then(from_utc_datetime)
                .map_err(|e| e.to_string())
        }
        _ => Err("schedule field out of range".to_owned()),
    };
    match marker {
        Ok(candidate) => Ok(candidate),
        Err(e) => {
            error!(
                year,
                month,
                day,
                hour_offset,
                minute_offset,
                second_offset,
                error = %e,
                "Could not construct monthly schedule marker"
            );
            Err(anyhow!("Could not construct monthly schedule marker"))
        }
    }
}

/// Calculate the next UTC DateTime that lands on the given day of the month
/// with provided hour/minute/second offsets.
/// - If the current month's scheduled time has already passed, schedules next month.
/// - Day is clamped to last day of month if requested day > maximum.
pub fn next_scheduled_month_mark(
    now: jiff::Timestamp,
    day_offset: u32,
    hour_offset: u32,
    minute_offset: u32,
    second_offset: u32,
) -> Result<jiff::Timestamp> {
    validate_day_and_time(day_offset, hour_offset, minute_offset, second_offset)?;

    let now_utc = to_utc_datetime(now);
    let year = i32::from(now_utc.year());
    let month = u32::from(now_utc.month().unsigned_abs());
    let candidate = build_month_mark(
        year,
        month,
        day_offset,
        hour_offset,
        minute_offset,
        second_offset,
    )?;
    if candidate > now {
        return Ok(candidate);
    }

    let (next_year, next_month_value) = next_month(year, month);
    build_month_mark(
        next_year,
        next_month_value,
        day_offset,
        hour_offset,
        minute_offset,
        second_offset,
    )
}

/// Returns (delay, next_mark) for the next monthly occurrence.
pub fn next_scheduled_monthly_delay(
    _task_descriptor: &str,
    day_offset: u32,
    hour_offset: u32,
    minute_offset: u32,
    second_offset: u32,
) -> Result<(tokio::time::Duration, jiff::Timestamp)> {
    let now = Timestamp::now();
    let next_mark =
        next_scheduled_month_mark(now, day_offset, hour_offset, minute_offset, second_offset)?;

    let delay = std::time::Duration::try_from(next_mark.duration_since(now)).map_err(|e| {
        anyhow!(
            "Could not schedule job at next_scheduled_month_mark(). negative delay: {:?}",
            e
        )
    })?;

    Ok((delay, next_mark))
}

/// Schedules a task to run once per month, at a specific
/// day+hour+minute+second offset (e.g., 10th day 02:15:30 UTC every month).
/// Day is clamped to last day of month if too high.
pub async fn schedule_task_every_month_at<F, Fut>(
    state: Arc<ServerState>,
    task: F,
    task_descriptor: String,
    day_offset: u32,
    hour_offset: u32,
    minute_offset: u32,
    second_offset: u32,
) -> Result<()>
where
    F: Fn(Arc<ServerState>) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    let mut initialized = false;
    let mut scheduled_run_time: Option<jiff::Timestamp> = None;
    loop {
        let (delay, next_mark) = match next_scheduled_monthly_delay(
            &task_descriptor,
            day_offset,
            hour_offset,
            minute_offset,
            second_offset,
        ) {
            Ok((delay, next_mark)) => (delay, next_mark),
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
                ?delay,
                delay_human = %format_duration(delay),
                "Scheduled task initialized"
            );
            initialized = true;
        }

        let this_run_time = match scheduled_run_time {
            Some(scheduled_run_time) => scheduled_run_time,
            None => next_mark,
        };

        tokio::time::sleep(delay).await;

        let start = tokio::time::Instant::now();
        task(Arc::clone(&state)).await;
        let elapsed = start.elapsed();

        let next_run_time = match next_scheduled_month_mark(
            this_run_time,
            day_offset,
            hour_offset,
            minute_offset,
            second_offset,
        ) {
            Ok(next_run_time) => next_run_time,
            Err(e) => {
                error!(
                    task_name = %task_descriptor,
                    error = ?e,
                    "Could not calculate following monthly scheduled time"
                );
                continue;
            }
        };
        let next_delay = match std::time::Duration::try_from(
            next_run_time.duration_since(Timestamp::now()),
        ) {
            Ok(next_delay) => next_delay,
            Err(e) => {
                error!(task_name = %task_descriptor, error = ?e, "Scheduled task next delay was negative");
                std::time::Duration::from_secs(2_592_000)
            }
        };

        info!(
            task_name = %task_descriptor,
            next_run_time = %next_run_time,
            duration=?elapsed,
            next_delay_human = %format_duration(next_delay),
            "Scheduled task ran"
        );

        scheduled_run_time = Some(next_run_time);
    }
}
