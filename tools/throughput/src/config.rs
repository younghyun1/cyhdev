//! Checked-in workload, environment, and threshold configuration.

use std::{collections::BTreeMap, fs::File, io::Read, path::Path};

use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{
    config_validation::{validate_environment, validate_thresholds, validate_workload},
    error::{HarnessError, HarnessResult},
};

const MAX_CONFIG_BYTES: u64 = 1024 * 1024;
const MAX_REPORT_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkloadConfig {
    pub schema_version: u32,
    pub name: String,
    pub iterations: u64,
    pub warmup_iterations: u64,
    pub concurrency: usize,
    pub timeout_ms: u64,
    pub max_response_bytes: usize,
    pub requests: Vec<RequestSpec>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RequestSpec {
    pub name: String,
    pub method: String,
    pub path: String,
    pub expected_status: u16,
    pub weight: u32,
    pub fixture_work_units: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EnvironmentConfig {
    pub schema_version: u32,
    pub label: String,
    pub power_profile: String,
    pub build_profile: String,
    pub configuration: BTreeMap<String, String>,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ThresholdConfig {
    pub schema_version: u32,
    pub workload_name: String,
    pub workload_digest: String,
    pub environment_label: String,
    pub environment_digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_environment_digest: Option<String>,
    pub compiled_profile: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub implementation_digest: Option<String>,
    pub executor_kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_address: Option<String>,
    pub minimum_requests_per_second: f64,
    pub maximum_error_rate_percent: f64,
    pub maximum_p50_latency_us: u64,
    pub maximum_p95_latency_us: u64,
    pub maximum_p99_latency_us: u64,
}

pub fn load_workload(path: &Path) -> HarnessResult<(WorkloadConfig, String)> {
    let (workload, digest) = load_json(path)?;
    validate_workload(path, &workload)?;
    Ok((workload, digest))
}

pub fn load_environment(path: &Path) -> HarnessResult<(EnvironmentConfig, String)> {
    let (environment, _raw_digest) = load_json(path)?;
    validate_environment(path, &environment)?;
    let digest = environment_config_digest(&environment)?;
    Ok((environment, digest))
}

pub fn environment_config_digest(environment: &EnvironmentConfig) -> HarnessResult<String> {
    let bytes = serde_json::to_vec(environment).map_err(|source| HarnessError::Json {
        path: Path::new("<declared-environment>").to_path_buf(),
        source,
    })?;
    Ok(fnv1a64_hex(&bytes))
}

pub fn load_thresholds(path: &Path) -> HarnessResult<(ThresholdConfig, String)> {
    let (thresholds, digest) = load_json(path)?;
    validate_thresholds(path, &thresholds)?;
    Ok((thresholds, digest))
}

pub fn load_report<T: DeserializeOwned>(path: &Path) -> HarnessResult<T> {
    let bytes = read_bounded(path, MAX_REPORT_BYTES)?;
    serde_json::from_slice(&bytes).map_err(|source| HarnessError::Json {
        path: path.to_path_buf(),
        source,
    })
}

fn load_json<T: DeserializeOwned>(path: &Path) -> HarnessResult<(T, String)> {
    let bytes = read_bounded(path, MAX_CONFIG_BYTES)?;
    let value = serde_json::from_slice(&bytes).map_err(|source| HarnessError::Json {
        path: path.to_path_buf(),
        source,
    })?;
    Ok((value, fnv1a64_hex(&bytes)))
}

fn read_bounded(path: &Path, maximum_bytes: u64) -> HarnessResult<Vec<u8>> {
    let file = File::open(path).map_err(|source| HarnessError::Io {
        operation: "open",
        path: path.to_path_buf(),
        source,
    })?;
    let metadata = file.metadata().map_err(|source| HarnessError::Io {
        operation: "inspect",
        path: path.to_path_buf(),
        source,
    })?;
    if metadata.len() > maximum_bytes {
        return Err(HarnessError::InputTooLarge {
            path: path.to_path_buf(),
            maximum_bytes,
        });
    }

    // The limit on the reader closes the metadata/read race if a file grows
    // after inspection, while still allowing an exact one-byte overflow check.
    let mut bytes = Vec::new();
    file.take(maximum_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|source| HarnessError::Io {
            operation: "read",
            path: path.to_path_buf(),
            source,
        })?;
    let byte_count = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    if byte_count > maximum_bytes {
        return Err(HarnessError::InputTooLarge {
            path: path.to_path_buf(),
            maximum_bytes,
        });
    }
    Ok(bytes)
}

pub(crate) fn fnv1a64_hex(bytes: &[u8]) -> String {
    let mut digest = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        digest ^= u64::from(*byte);
        digest = digest.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("fnv1a64:{digest:016x}")
}

#[cfg(test)]
mod tests {
    use super::{MAX_CONFIG_BYTES, read_bounded};
    use crate::error::HarnessError;

    #[test]
    fn rejects_oversized_json_before_deserialization() -> Result<(), String> {
        let file = tempfile::NamedTempFile::new()
            .map_err(|error| format!("could not create temporary input: {error}"))?;
        file.as_file()
            .set_len(MAX_CONFIG_BYTES.saturating_add(1))
            .map_err(|error| format!("could not size temporary input: {error}"))?;

        match read_bounded(file.path(), MAX_CONFIG_BYTES) {
            Err(HarnessError::InputTooLarge { maximum_bytes, .. })
                if maximum_bytes == MAX_CONFIG_BYTES =>
            {
                Ok(())
            }
            Err(error) => Err(format!("expected size rejection, got {error}")),
            Ok(_) => Err("oversized input was accepted".to_owned()),
        }
    }
}
