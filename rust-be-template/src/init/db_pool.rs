//! Runtime PostgreSQL pool: explicit size, per-session time bounds, and cheap checkout checks.
//!
//! The migration runner connects with the plain URL, so these session limits apply only
//! to pooled request and job connections. Migrations and backfills keep PostgreSQL's
//! unbounded defaults because they run once, before serving, on their own connection.

use std::time::Duration;

use diesel::result::{DatabaseErrorKind, Error as DieselError};
use diesel_async::{
    AsyncPgConnection,
    pooled_connection::{
        AsyncDieselConnectionManager, ManagerConfig, PoolableConnection, RecyclingMethod, bb8::Pool,
    },
};

use super::db_config::percent_encode;

/// Pool size when `DB_POOL_MAX_SIZE` is unset. One process serves the site; the
/// bound leaves most of PostgreSQL's default 100 connections for maintenance,
/// migrations, and administrative sessions.
pub const DEFAULT_POOL_MAX_SIZE: u32 = 24;
const MAX_POOL_MAX_SIZE: u32 = 256;
const POOL_MIN_IDLE: u32 = 4;
const CHECKOUT_TIMEOUT: Duration = Duration::from_secs(2);

/// Longest single statement on a pooled connection. Request paths are indexed and
/// bounded; a statement exceeding this is a plan or lock regression, not work to wait on.
pub const STATEMENT_TIMEOUT: Duration = Duration::from_secs(30);
/// Longest wait for a row or table lock, so one stuck writer cannot drain the pool.
pub const LOCK_TIMEOUT: Duration = Duration::from_secs(5);
/// Longest idle gap inside an open transaction; repositories never await network
/// work inside a transaction, so a longer gap means an abandoned connection.
pub const IDLE_IN_TRANSACTION_TIMEOUT: Duration = Duration::from_secs(60);

/// Pool settings read once at startup.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DbPoolSettings {
    pub max_size: u32,
}

impl DbPoolSettings {
    /// Reads `DB_POOL_MAX_SIZE`; an invalid value aborts startup instead of silently
    /// falling back.
    pub fn from_env() -> anyhow::Result<Self> {
        match std::env::var("DB_POOL_MAX_SIZE") {
            Ok(value) => Self::parse(Some(&value)),
            Err(std::env::VarError::NotPresent) => Self::parse(None),
            Err(std::env::VarError::NotUnicode(_)) => {
                Err(anyhow::anyhow!("DB_POOL_MAX_SIZE must contain UTF-8"))
            }
        }
    }

    fn parse(configured: Option<&str>) -> anyhow::Result<Self> {
        let max_size = match configured.map(str::trim) {
            None | Some("") => DEFAULT_POOL_MAX_SIZE,
            Some(value) => value
                .parse::<u32>()
                .map_err(|error| anyhow::anyhow!("DB_POOL_MAX_SIZE must be an integer: {error}"))?,
        };
        if !(1..=MAX_POOL_MAX_SIZE).contains(&max_size) {
            return Err(anyhow::anyhow!(
                "DB_POOL_MAX_SIZE must be between 1 and {MAX_POOL_MAX_SIZE}"
            ));
        }
        Ok(Self { max_size })
    }

    pub fn min_idle(self) -> u32 {
        POOL_MIN_IDLE.min(self.max_size)
    }
}

/// PostgreSQL startup `options` that set the session time bounds.
pub fn session_options() -> String {
    format!(
        "-c statement_timeout={} -c lock_timeout={} -c idle_in_transaction_session_timeout={}",
        STATEMENT_TIMEOUT.as_millis(),
        LOCK_TIMEOUT.as_millis(),
        IDLE_IN_TRANSACTION_TIMEOUT.as_millis()
    )
}

/// Appends the encoded session `options` parameter to a connection URL.
///
/// A URL that already sets `options` is rejected rather than merged, because two
/// `options` parameters would silently let one override the pool's time bounds.
pub fn with_session_options(url: &str) -> anyhow::Result<String> {
    let query = url.split_once('?').map(|(_, query)| query);
    let already_set = query
        .into_iter()
        .flat_map(|query| query.split('&'))
        .any(|pair| pair.split_once('=').map_or(pair, |(key, _)| key) == "options");
    if already_set {
        return Err(anyhow::anyhow!(
            "Database URL must not set `options`; the pool sets session timeouts"
        ));
    }
    let separator = if query.is_some() { '&' } else { '?' };
    Ok(format!(
        "{url}{separator}options={}",
        percent_encode(&session_options())
    ))
}

/// Builds the request/job pool from a URL that already carries [`with_session_options`].
///
/// bb8 still validates every checkout, but with a local check instead of a `SELECT 1`
/// round trip: diesel-async reports a connection as broken when its transaction
/// manager is broken or the tokio-postgres client has closed, which covers a
/// connection whose server went away while it sat idle in the pool.
pub async fn build_pool(
    url: &str,
    settings: DbPoolSettings,
) -> anyhow::Result<Pool<AsyncPgConnection>> {
    let mut manager_config = ManagerConfig::<AsyncPgConnection>::default();
    manager_config.recycling_method = RecyclingMethod::CustomFunction(Box::new(|connection| {
        let broken = connection.is_broken();
        Box::pin(async move {
            if broken {
                Err(DieselError::DatabaseError(
                    DatabaseErrorKind::ClosedConnection,
                    Box::new("pooled connection is closed or mid-transaction".to_owned()),
                ))
            } else {
                Ok(())
            }
        })
    }));
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_config(
        url.to_owned(),
        manager_config,
    );
    Pool::builder()
        .max_size(settings.max_size)
        .min_idle(Some(settings.min_idle()))
        .test_on_check_out(true)
        .connection_timeout(CHECKOUT_TIMEOUT)
        .build(manager)
        .await
        .map_err(|error| anyhow::anyhow!("Failed to build connection pool: {error}"))
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_POOL_MAX_SIZE, DbPoolSettings, session_options, with_session_options};

    #[test]
    fn pool_size_defaults_and_rejects_out_of_range_values() {
        assert_eq!(
            DbPoolSettings::parse(None).ok(),
            Some(DbPoolSettings {
                max_size: DEFAULT_POOL_MAX_SIZE
            })
        );
        assert_eq!(
            DbPoolSettings::parse(Some(" 12 "))
                .map(|settings| settings.max_size)
                .ok(),
            Some(12)
        );
        assert_eq!(
            DbPoolSettings::parse(Some("2"))
                .map(DbPoolSettings::min_idle)
                .ok(),
            Some(2)
        );
        for invalid in ["0", "257", "-1", "twenty"] {
            assert!(DbPoolSettings::parse(Some(invalid)).is_err(), "{invalid}");
        }
    }

    #[test]
    fn session_options_are_encoded_as_one_query_parameter() -> anyhow::Result<()> {
        assert_eq!(
            session_options(),
            "-c statement_timeout=30000 -c lock_timeout=5000 -c idle_in_transaction_session_timeout=60000"
        );
        let encoded = "options=-c%20statement_timeout%3D30000%20-c%20lock_timeout%3D5000%20-c%20idle_in_transaction_session_timeout%3D60000";
        assert_eq!(
            with_session_options("postgres://u:p@localhost:5432/db")?,
            format!("postgres://u:p@localhost:5432/db?{encoded}")
        );
        assert_eq!(
            with_session_options("postgres://u:p@/db?host=%2Frun%2Fpostgresql")?,
            format!("postgres://u:p@/db?host=%2Frun%2Fpostgresql&{encoded}")
        );
        Ok(())
    }

    #[test]
    fn caller_supplied_options_are_rejected() {
        assert!(with_session_options("postgres://u:p@h/db?options=-c%20x%3D1").is_err());
        assert!(with_session_options("postgres://u:p@h/db?sslmode=disable&options").is_err());
        assert!(with_session_options("postgres://u:p@h/db?application_name=options").is_ok());
    }
}
