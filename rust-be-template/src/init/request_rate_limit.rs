//! Startup-only admission of a finite request policy for isolated optimization fixtures.

use std::{net::IpAddr, time::Duration};

use serde_derive::Deserialize;

use super::state::DeploymentEnvironment;
use crate::util::request_rate_limit::RequestRatePolicy;

const OPTIMIZATION_POLICY: &str = include_str!("../../../tools/optimization/request-limits.json");
const OPTIMIZATION_PROFILE: &str = "optimization-v1";

/// Resolves the policy before certificates, database connections, or listeners are opened.
pub(super) fn from_environment(bind: IpAddr) -> anyhow::Result<RequestRatePolicy> {
    let profile = optional_env("CYHDEV_OPT_REQUEST_LIMITS")?;
    if profile.is_none() {
        return Ok(RequestRatePolicy::HISTORICAL);
    }
    let disposable = optional_env("CYHDEV_OPT_DISPOSABLE")?;
    let origin = optional_env("PUBLIC_APP_ORIGIN")?;
    let database = optional_env("DB_URL")?;
    let database_host = optional_env("DB_HOST")?;
    let trusted_hops = optional_env("TRUSTED_PROXY_HOPS")?;
    let trusted_networks = optional_env("TRUSTED_PROXY_CIDRS")?;
    select(RequestLimitInputs {
        profile: profile.as_deref(),
        deployment: DeploymentEnvironment::from_env()?,
        bind,
        disposable: disposable.as_deref(),
        origin: origin.as_deref(),
        database: database.as_deref(),
        database_host: database_host.as_deref(),
        trusted_hops: trusted_hops.as_deref(),
        trusted_networks: trusted_networks.as_deref(),
    })
}

/// Pure inputs keep startup rejection tests independent of process environment mutation.
struct RequestLimitInputs<'a> {
    profile: Option<&'a str>,
    deployment: DeploymentEnvironment,
    bind: IpAddr,
    disposable: Option<&'a str>,
    origin: Option<&'a str>,
    database: Option<&'a str>,
    database_host: Option<&'a str>,
    trusted_hops: Option<&'a str>,
    trusted_networks: Option<&'a str>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OptimizationPolicy {
    schema_version: u8,
    profile: String,
    refill_interval_micros: u64,
    burst_size: u32,
}

fn select(inputs: RequestLimitInputs<'_>) -> anyhow::Result<RequestRatePolicy> {
    match inputs.profile {
        None => return Ok(RequestRatePolicy::HISTORICAL),
        Some(OPTIMIZATION_PROFILE) => {}
        Some(_) => return Err(anyhow::anyhow!("unknown CYHDEV_OPT_REQUEST_LIMITS profile")),
    }
    if inputs.deployment != DeploymentEnvironment::Local
        || !inputs.bind.to_canonical().is_loopback()
        || inputs.disposable != Some("1")
    {
        return Err(anyhow::anyhow!(
            "optimization request limits require local deployment, a loopback listener, and disposable fixtures"
        ));
    }
    validate_origin(inputs.origin)?;
    validate_database(inputs.database, inputs.database_host)?;
    if !matches!(inputs.trusted_hops, None | Some("") | Some("0"))
        || inputs
            .trusted_networks
            .is_some_and(|networks| !networks.is_empty())
    {
        return Err(anyhow::anyhow!(
            "optimization request limits do not permit trusted proxy configuration"
        ));
    }
    shared_policy()
}

fn shared_policy() -> anyhow::Result<RequestRatePolicy> {
    let policy: OptimizationPolicy = match serde_json::from_str(OPTIMIZATION_POLICY) {
        Ok(policy) => policy,
        Err(_) => {
            return Err(anyhow::anyhow!(
                "invalid embedded optimization request policy"
            ));
        }
    };
    if policy.schema_version != 1 || policy.profile != OPTIMIZATION_PROFILE {
        return Err(anyhow::anyhow!(
            "unsupported embedded optimization request policy"
        ));
    }
    RequestRatePolicy::bounded(
        Duration::from_micros(policy.refill_interval_micros),
        policy.burst_size,
    )
}

fn validate_origin(origin: Option<&str>) -> anyhow::Result<()> {
    let origin = match origin.and_then(|origin| reqwest::Url::parse(origin).ok()) {
        Some(origin) => origin,
        None => {
            return Err(anyhow::anyhow!(
                "optimization request limits require a loopback public origin"
            ));
        }
    };
    if !matches!(origin.scheme(), "http" | "https")
        || !literal_loopback(&origin)
        || !origin.username().is_empty()
        || origin.password().is_some()
        || origin.path() != "/"
        || origin.query().is_some()
        || origin.fragment().is_some()
    {
        return Err(anyhow::anyhow!(
            "optimization request limits require an exact loopback public origin"
        ));
    }
    Ok(())
}

fn validate_database(database: Option<&str>, database_host: Option<&str>) -> anyhow::Result<()> {
    let database = match database.and_then(|database| reqwest::Url::parse(database).ok()) {
        Some(database) => database,
        None => {
            return Err(anyhow::anyhow!(
                "optimization request limits require a fixture database URL"
            ));
        }
    };
    let name = match database.path().strip_prefix('/') {
        Some(name) => name,
        None => {
            return Err(anyhow::anyhow!(
                "optimization request limits require a fixture database path"
            ));
        }
    };
    if !matches!(database.scheme(), "postgres" | "postgresql")
        || !literal_loopback(&database)
        || !name.starts_with("cyhdev_optimization_")
        || name.len() <= "cyhdev_optimization_".len()
        || name.len() > 63
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        || database.query().is_some()
        || database.fragment().is_some()
        || database_host.is_some_and(|host| host.starts_with('/'))
    {
        return Err(anyhow::anyhow!(
            "optimization request limits require a loopback cyhdev_optimization_* database without overrides"
        ));
    }
    Ok(())
}

/// Numeric loopback addresses cannot resolve through an externally controlled hostname.
fn literal_loopback(url: &reqwest::Url) -> bool {
    match url.host_str().and_then(|host| {
        host.trim_start_matches('[')
            .trim_end_matches(']')
            .parse::<IpAddr>()
            .ok()
    }) {
        Some(address) => address.to_canonical().is_loopback(),
        None => false,
    }
}

fn optional_env(name: &str) -> anyhow::Result<Option<String>> {
    match std::env::var(name) {
        Ok(value) => Ok(Some(value)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => Err(anyhow::anyhow!("{name} must be UTF-8")),
    }
}

#[cfg(test)]
#[path = "request_rate_limit_tests.rs"]
mod tests;
