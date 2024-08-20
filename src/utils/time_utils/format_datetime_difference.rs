use chrono::{DateTime, TimeDelta, Utc};

pub fn format_dt_difference(start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> String {
    let differential: TimeDelta = end_time - start_time;
    format!(
        "{} days, {} hours, {} minutes, {} seconds",
        differential.num_days(),
        differential.num_hours() % 24,
        differential.num_minutes() % 60,
        differential.num_seconds() % 60
    )
}
