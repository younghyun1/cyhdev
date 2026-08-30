//! Fail-closed validation for the destructive PostgreSQL integration harness.

use std::{env, net::IpAddr, process::Command};

use crate::{TaskError, TaskResult};

const DISPOSABLE_MAINTENANCE_DATABASE: &str = "cyhdev_test_maintenance";
const REMOTE_OVERRIDE_ENV: &str = "TEST_DATABASE_ALLOW_REMOTE_CI";

pub(crate) fn validate() -> TaskResult<()> {
    let database_url = match env::var("TEST_DATABASE_URL") {
        Ok(value) if !value.trim().is_empty() => value,
        Ok(_) | Err(env::VarError::NotPresent) => {
            return Err(TaskError("TEST_DATABASE_URL is required".to_owned()));
        }
        Err(env::VarError::NotUnicode(_)) => {
            return Err(TaskError("TEST_DATABASE_URL must contain valid UTF-8".to_owned()));
        }
    };
    if !database_url.starts_with("postgres://") && !database_url.starts_with("postgresql://") {
        return Err(TaskError(
            "TEST_DATABASE_URL must use postgres:// or postgresql://".to_owned(),
        ));
    }

    let output = Command::new("psql")
        .args([
            "--no-psqlrc",
            "--quiet",
            "--tuples-only",
            "--no-align",
            "--field-separator=|",
            "--set=ON_ERROR_STOP=1",
            "--command",
            "SELECT current_setting('server_version_num'), current_database(), COALESCE(inet_server_addr()::text, '')",
        ])
        .env("PGDATABASE", &database_url)
        .output()
        .map_err(|error| TaskError(format!("failed to execute psql for test database validation: {error}")))?;
    drop(database_url);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(TaskError(format!(
            "test database validation query failed: {}",
            stderr.trim()
        )));
    }
    let stdout = String::from_utf8(output.stdout)
        .map_err(|_| TaskError("test database validation returned non-UTF-8 output".to_owned()))?;
    let fields = stdout.trim().split('|').map(str::trim).collect::<Vec<_>>();
    let (version, database_name, server_address) = match fields.as_slice() {
        [version, database_name, server_address] => (*version, *database_name, *server_address),
        _ => return Err(TaskError("test database validation returned an unexpected row".to_owned())),
    };
    let version = version.parse::<u32>()
        .map_err(|_| TaskError("PostgreSQL returned an invalid server_version_num".to_owned()))?;
    if !(180_000..190_000).contains(&version) {
        return Err(TaskError(format!("PostgreSQL 18 is required; server_version_num is {version}")));
    }
    if database_name != DISPOSABLE_MAINTENANCE_DATABASE {
        return Err(TaskError(format!(
            "TEST_DATABASE_URL must select the disposable maintenance database {DISPOSABLE_MAINTENANCE_DATABASE:?}"
        )));
    }
    validate_server_address(server_address)
}

fn validate_server_address(server_address: &str) -> TaskResult<()> {
    if server_address.is_empty() {
        return Ok(());
    }
    let address = server_address.parse::<IpAddr>()
        .map_err(|_| TaskError("PostgreSQL returned an invalid server address".to_owned()))?;
    if address.is_loopback() {
        return Ok(());
    }
    let override_enabled = matches!(env::var(REMOTE_OVERRIDE_ENV), Ok(value) if value == "1");
    let ci_enabled = matches!(env::var("CI"), Ok(value) if value == "1" || value.eq_ignore_ascii_case("true"));
    if override_enabled && ci_enabled {
        Ok(())
    } else {
        Err(TaskError(format!(
            "remote PostgreSQL server {address} is forbidden; CI must set CI=true and {REMOTE_OVERRIDE_ENV}=1 explicitly"
        )))
    }
}
