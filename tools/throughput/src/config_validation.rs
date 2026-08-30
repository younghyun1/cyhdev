//! Bounds and semantic validation for all harness inputs.

use std::{net::SocketAddr, path::Path};

use crate::{
    config::{EnvironmentConfig, RequestSpec, ThresholdConfig, WorkloadConfig},
    error::{HarnessError, HarnessResult},
};

const CONFIG_SCHEMA_VERSION: u32 = 1;
const THRESHOLD_SCHEMA_VERSION: u32 = 2;
const MAX_ITERATIONS: u64 = 1_000_000;
const MAX_WARMUP_ITERATIONS: u64 = 100_000;
const MAX_CONCURRENCY: usize = 64;
const MAX_REQUESTS: usize = 256;
const MAX_TOTAL_WEIGHT: u64 = 1_000_000;
const MAX_FIXTURE_WORK_UNITS: u64 = 1_000_000;
const MAX_RESPONSE_BYTES: usize = 16 * 1024 * 1024;
const MAX_AGGREGATE_RESPONSE_BYTES: usize = 64 * 1024 * 1024;

pub fn validate_workload(path: &Path, workload: &WorkloadConfig) -> HarnessResult<()> {
    validate_schema(path, workload.schema_version, CONFIG_SCHEMA_VERSION)?;
    validate_text(path, "workload name", &workload.name, 128)?;
    if !(1..=MAX_ITERATIONS).contains(&workload.iterations) {
        return invalid(path, format!("iterations must be in 1..={MAX_ITERATIONS}"));
    }
    if workload.warmup_iterations > MAX_WARMUP_ITERATIONS {
        return invalid(
            path,
            format!("warmup_iterations must be <= {MAX_WARMUP_ITERATIONS}"),
        );
    }
    if !(1..=MAX_CONCURRENCY).contains(&workload.concurrency) {
        return invalid(
            path,
            format!("concurrency must be in 1..={MAX_CONCURRENCY}"),
        );
    }
    if !(1..=60_000).contains(&workload.timeout_ms) {
        return invalid(path, "timeout_ms must be in 1..=60000".to_owned());
    }
    if !(1_024..=MAX_RESPONSE_BYTES).contains(&workload.max_response_bytes) {
        return invalid(
            path,
            format!("max_response_bytes must be in 1024..={MAX_RESPONSE_BYTES}"),
        );
    }
    let aggregate_response_bytes = workload
        .concurrency
        .checked_mul(workload.max_response_bytes)
        .ok_or_else(|| HarnessError::Configuration {
            path: path.to_path_buf(),
            detail: "concurrency multiplied by max_response_bytes overflowed".to_owned(),
        })?;
    if aggregate_response_bytes > MAX_AGGREGATE_RESPONSE_BYTES {
        return invalid(
            path,
            format!(
                "concurrency multiplied by max_response_bytes must be <= {MAX_AGGREGATE_RESPONSE_BYTES}"
            ),
        );
    }
    if workload.requests.is_empty() || workload.requests.len() > MAX_REQUESTS {
        return invalid(
            path,
            format!("requests must contain 1..={MAX_REQUESTS} entries"),
        );
    }

    let mut total_weight = 0_u64;
    for request in &workload.requests {
        validate_request(path, request)?;
        total_weight = total_weight.saturating_add(u64::from(request.weight));
    }
    if total_weight > MAX_TOTAL_WEIGHT {
        return invalid(
            path,
            format!("combined request weight must be <= {MAX_TOTAL_WEIGHT}"),
        );
    }
    Ok(())
}

pub fn validate_environment(path: &Path, environment: &EnvironmentConfig) -> HarnessResult<()> {
    validate_schema(path, environment.schema_version, CONFIG_SCHEMA_VERSION)?;
    validate_text(path, "environment label", &environment.label, 128)?;
    validate_text(path, "power profile", &environment.power_profile, 128)?;
    validate_text(path, "build profile", &environment.build_profile, 128)?;
    if environment.configuration.is_empty() || environment.configuration.len() > 64 {
        return invalid(
            path,
            "environment configuration must contain 1..=64 entries".to_owned(),
        );
    }
    for (key, value) in &environment.configuration {
        validate_text(path, "environment configuration key", key, 64)?;
        validate_text(path, "environment configuration value", value, 256)?;
    }
    if environment.notes.len() > 32 {
        return invalid(path, "environment notes must contain <= 32 entries".to_owned());
    }
    for note in &environment.notes {
        validate_text(path, "environment note", note, 512)?;
    }
    Ok(())
}

pub fn validate_thresholds(path: &Path, thresholds: &ThresholdConfig) -> HarnessResult<()> {
    validate_schema(path, thresholds.schema_version, THRESHOLD_SCHEMA_VERSION)?;
    validate_text(path, "threshold workload_name", &thresholds.workload_name, 128)?;
    validate_digest(path, "workload_digest", &thresholds.workload_digest)?;
    validate_text(path, "environment_label", &thresholds.environment_label, 128)?;
    validate_digest(path, "environment_digest", &thresholds.environment_digest)?;
    if let Some(digest) = &thresholds.observed_environment_digest {
        validate_digest(path, "observed_environment_digest", digest)?;
    }
    validate_text(path, "compiled_profile", &thresholds.compiled_profile, 32)?;
    if !matches!(thresholds.executor_kind.as_str(), "fixture" | "http") {
        return invalid(
            path,
            "executor_kind must be `fixture` or `http`".to_owned(),
        );
    }
    if thresholds.executor_kind == "http" && thresholds.observed_environment_digest.is_none() {
        return invalid(
            path,
            "HTTP thresholds must pin observed_environment_digest".to_owned(),
        );
    }
    if thresholds.executor_kind == "http" {
        let implementation = thresholds.implementation_digest.as_ref().ok_or_else(|| {
            HarnessError::Configuration { path: path.to_path_buf(), detail: "HTTP thresholds must pin implementation_digest".to_owned() }
        })?;
        validate_sha256_digest(path, "implementation_digest", implementation)?;
        let address = thresholds.resolved_address.as_ref().ok_or_else(|| {
            HarnessError::Configuration { path: path.to_path_buf(), detail: "HTTP thresholds must pin resolved_address".to_owned() }
        })?;
        if address.parse::<SocketAddr>().is_err() {
            return invalid(path, "resolved_address must be a socket address".to_owned());
        }
    } else if thresholds.implementation_digest.is_some() || thresholds.resolved_address.is_some() {
        return invalid(path, "fixture thresholds must omit implementation_digest and resolved_address".to_owned());
    }
    if !thresholds.minimum_requests_per_second.is_finite()
        || thresholds.minimum_requests_per_second <= 0.0
    {
        return invalid(
            path,
            "minimum_requests_per_second must be finite and positive".to_owned(),
        );
    }
    if !thresholds.maximum_error_rate_percent.is_finite()
        || !(0.0..=100.0).contains(&thresholds.maximum_error_rate_percent)
    {
        return invalid(
            path,
            "maximum_error_rate_percent must be finite and in 0..=100".to_owned(),
        );
    }
    if thresholds.maximum_p50_latency_us == 0
        || thresholds.maximum_p50_latency_us > thresholds.maximum_p95_latency_us
        || thresholds.maximum_p95_latency_us > thresholds.maximum_p99_latency_us
    {
        return invalid(
            path,
            "latency limits must be positive and ordered p50 <= p95 <= p99".to_owned(),
        );
    }
    Ok(())
}

fn validate_request(path: &Path, request: &RequestSpec) -> HarnessResult<()> {
    validate_text(path, "request name", &request.name, 128)?;
    if !matches!(request.method.as_str(), "GET" | "HEAD") {
        return invalid(
            path,
            format!("request `{}` must use GET or HEAD", request.name),
        );
    }
    if !request.path.starts_with('/')
        || !request.path.is_ascii()
        || request
            .path
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte.is_ascii_whitespace())
        || request.path.chars().count() > 2_048
    {
        return invalid(
            path,
            format!("request `{}` has an invalid path", request.name),
        );
    }
    if !(100..=599).contains(&request.expected_status) {
        return invalid(
            path,
            format!("request `{}` has an invalid expected_status", request.name),
        );
    }
    if request.weight == 0 {
        return invalid(
            path,
            format!("request `{}` must have a positive weight", request.name),
        );
    }
    if !(1..=MAX_FIXTURE_WORK_UNITS).contains(&request.fixture_work_units) {
        return invalid(
            path,
            format!(
                "request `{}` fixture_work_units must be in 1..={MAX_FIXTURE_WORK_UNITS}",
                request.name
            ),
        );
    }
    Ok(())
}

fn validate_schema(path: &Path, version: u32, expected: u32) -> HarnessResult<()> {
    if version == expected {
        Ok(())
    } else {
        invalid(
            path,
            format!("schema_version must be {expected}, got {version}"),
        )
    }
}

fn validate_digest(path: &Path, field: &str, value: &str) -> HarnessResult<()> {
    let valid = value.len() == 25
        && value.starts_with("fnv1a64:")
        && value[9..]
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'));
    if valid {
        Ok(())
    } else {
        invalid(path, format!("{field} must be a lowercase fnv1a64 digest"))
    }
}

fn validate_sha256_digest(path: &Path, field: &str, value: &str) -> HarnessResult<()> {
    let valid = value.len() == 71
        && value.starts_with("sha256:")
        && value[7..].bytes().all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'));
    if valid { Ok(()) } else { invalid(path, format!("{field} must be a lowercase sha256 digest")) }
}

fn validate_text(path: &Path, field: &str, value: &str, max: usize) -> HarnessResult<()> {
    let length = value.chars().count();
    if value.trim().is_empty() || length > max {
        invalid(path, format!("{field} must contain 1..={max} characters"))
    } else {
        Ok(())
    }
}

fn invalid<T>(path: &Path, detail: String) -> HarnessResult<T> {
    Err(HarnessError::Configuration {
        path: path.to_path_buf(),
        detail,
    })
}
