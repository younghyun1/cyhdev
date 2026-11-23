//! UTC calendar helpers over `jiff` instants.
//!
//! The backend stores and schedules instants as `jiff::Timestamp`. Calendar
//! arithmetic (truncating to a day, building "the 3rd at 02:00") needs a civil
//! date-time, and every such computation here is defined in UTC, so these
//! helpers fix the offset to UTC instead of consulting the host time zone.

use jiff::{SignedDuration, Timestamp, civil, tz::Offset};

/// The UTC wall-clock reading of an instant.
pub fn to_utc_datetime(timestamp: Timestamp) -> civil::DateTime {
    Offset::UTC.to_datetime(timestamp)
}

/// The instant at which UTC wall clocks read `datetime`.
pub fn from_utc_datetime(datetime: civil::DateTime) -> Result<Timestamp, jiff::Error> {
    Offset::UTC.to_timestamp(datetime)
}

/// How long to sleep from `now` until `target`; zero when `target` is not in the future.
pub fn delay_until(now: Timestamp, target: Timestamp) -> std::time::Duration {
    let remaining: SignedDuration = target.duration_since(now);
    match std::time::Duration::try_from(remaining) {
        Ok(delay) => delay,
        // Negative: the target has already passed.
        Err(_) => std::time::Duration::ZERO,
    }
}

/// RFC 3339 text with an explicit `+00:00` offset, as user-facing messages
/// have always shown it, rather than jiff's default `Z` suffix.
pub fn rfc3339_utc(timestamp: Timestamp) -> String {
    timestamp.display_with_offset(Offset::UTC).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_utc_wall_clock() {
        let instant = Timestamp::from_second(1_700_000_000);
        let Ok(instant) = instant else {
            panic!("fixed instant is in range");
        };
        let civil = to_utc_datetime(instant);
        assert_eq!(civil.to_string(), "2023-11-14T22:13:20");
        assert!(matches!(from_utc_datetime(civil), Ok(value) if value == instant));
    }

    #[test]
    fn delay_is_zero_for_past_targets() {
        let Ok(now) = Timestamp::from_second(100) else {
            panic!("fixed instant is in range");
        };
        let Ok(earlier) = Timestamp::from_second(90) else {
            panic!("fixed instant is in range");
        };
        assert_eq!(delay_until(now, earlier), std::time::Duration::ZERO);
        assert_eq!(
            delay_until(earlier, now),
            std::time::Duration::from_secs(10)
        );
    }

    #[test]
    fn formats_with_numeric_utc_offset() {
        let Ok(instant) = Timestamp::from_second(1_700_000_000) else {
            panic!("fixed instant is in range");
        };
        assert_eq!(rfc3339_utc(instant), "2023-11-14T22:13:20+00:00");
    }
}
