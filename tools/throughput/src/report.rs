//! Stable JSON report schema and bounded report persistence.

use std::{collections::BTreeMap, fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::{
    config::{EnvironmentConfig, ThresholdConfig},
    error::{HarnessError, HarnessResult},
    executor::types::ExecutorKind,
    hardware::ObservedHardware,
};

pub const REPORT_SCHEMA_VERSION: u32 = 4;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ThroughputReport {
    pub schema_version: u32,
    pub workload: WorkloadMetadata,
    pub executor: ExecutorMetadata,
    pub environment: EnvironmentMetadata,
    pub hardware: ObservedHardware,
    pub configuration: RunConfiguration,
    pub metrics: Metrics,
    /// Absent only for a threshold-free calibration record.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thresholds: Option<ThresholdEvidence>,
    /// Absent only for a threshold-free calibration record.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verdict: Option<Verdict>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkloadMetadata {
    pub name: String,
    pub digest: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutorMetadata {
    pub kind: ExecutorKind,
    pub target: String,
    pub resolved_address: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EnvironmentMetadata {
    /// Digest of the checked-in declaration used by portable thresholds.
    pub digest: String,
    /// Digest including the observed hardware, kernel, compiler, and profile.
    pub observed_digest: String,
    pub declared: EnvironmentConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RunConfiguration {
    pub harness_version: String,
    pub compiled_profile: String,
    #[serde(default)]
    pub implementation_digest: String,
    pub iterations: u64,
    pub warmup_iterations: u64,
    pub concurrency: usize,
    pub timeout_ms: u64,
    pub max_response_bytes: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Metrics {
    pub elapsed_ns: u64,
    pub requests: u64,
    pub successes: u64,
    pub failures: u64,
    pub error_rate_percent: f64,
    pub throughput_requests_per_second: f64,
    pub p50_latency_us: u64,
    pub p95_latency_us: u64,
    pub p99_latency_us: u64,
    pub response_bytes: u64,
    pub response_checksum: String,
    pub failure_counts: BTreeMap<String, u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ThresholdEvidence {
    pub digest: String,
    pub configured: ThresholdConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Verdict {
    pub passed: bool,
    pub violations: Vec<String>,
}

pub fn write(path: &Path, report: &ThroughputReport) -> HarnessResult<()> {
    let bytes = serde_json::to_vec_pretty(report).map_err(|source| HarnessError::Json {
        path: path.to_path_buf(),
        source,
    })?;
    if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|source| HarnessError::Io {
            operation: "create report directory",
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
    fs::write(&temporary, bytes).map_err(|source| HarnessError::Io {
        operation: "write temporary report",
        path: temporary.clone(),
        source,
    })?;
    fs::rename(&temporary, path).map_err(|source| HarnessError::Io {
        operation: "replace throughput report",
        path: path.to_path_buf(),
        source,
    })
}

pub fn print_json<T: Serialize>(value: &T) -> HarnessResult<()> {
    let output = serde_json::to_string_pretty(value).map_err(|source| HarnessError::Json {
        path: Path::new("<stdout>").to_path_buf(),
        source,
    })?;
    println!("{output}");
    Ok(())
}

pub const fn compiled_profile() -> &'static str {
    if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    }
}
