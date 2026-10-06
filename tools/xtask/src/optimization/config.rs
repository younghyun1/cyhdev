//! Bounded, explicit inputs for an isolated optimization campaign.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::{TaskError, TaskResult};

use super::files;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Config {
    pub schema_version: u8,
    pub name: String,
    pub target_cpu: String,
    pub runtime_directory: PathBuf,
    pub campaign: PathBuf,
    pub training_command: Vec<String>,
    pub benchmark_command: Vec<String>,
    pub timeout_seconds: u64,
    pub maximum_latency_ratio: f64,
    pub minimum_throughput_ratio: f64,
}

impl Config {
    /// Resolve private inputs without reading credentials or backend `.env` files.
    pub fn read(root: &Path, path: &Path) -> TaskResult<Self> {
        let mut config: Self = files::read_json(path, 1024 * 1024)?;
        if config.schema_version != 1 {
            return Err(TaskError(
                "optimization config schema_version must be 1".into(),
            ));
        }
        token(&config.name)?;
        token(&config.target_cpu)?;
        command(&config.training_command)?;
        command(&config.benchmark_command)?;
        if !(1..=7200).contains(&config.timeout_seconds)
            || !config.maximum_latency_ratio.is_finite()
            || !(1.0..=1.25).contains(&config.maximum_latency_ratio)
            || !config.minimum_throughput_ratio.is_finite()
            || !(0.8..=1.2).contains(&config.minimum_throughput_ratio)
        {
            return Err(TaskError("invalid timeout or comparison budget".into()));
        }
        config.runtime_directory = match resolve(root, &config.runtime_directory) {
            Ok(runtime) => runtime,
            Err(error) => {
                return Err(TaskError(format!(
                    "configured fixture runtime is unavailable: {error}; run ./build_pgo_and_bolt.sh without a config to prepare fresh fixtures, or recreate the manually managed runtime"
                )));
            }
        };
        config.campaign = resolve(root, &config.campaign)?;
        if !config.runtime_directory.is_dir() || !config.campaign.is_file() {
            return Err(TaskError(
                "runtime_directory and campaign must exist".into(),
            ));
        }
        if config.runtime_directory.starts_with(root) {
            return Err(TaskError(
                "runtime_directory must be outside the source checkout".into(),
            ));
        }
        Ok(config)
    }
}

fn resolve(root: &Path, path: &Path) -> TaskResult<PathBuf> {
    match root.join(path).canonicalize() {
        Ok(path) => Ok(path),
        Err(error) => Err(TaskError(format!(
            "cannot resolve {}: {error}",
            path.display()
        ))),
    }
}

pub(super) fn token(value: &str) -> TaskResult<()> {
    if value.is_empty()
        || value.len() > 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(TaskError(
            "expected an ASCII name or CPU token of 1-64 characters".into(),
        ));
    }
    Ok(())
}

fn command(arguments: &[String]) -> TaskResult<()> {
    if arguments.is_empty()
        || arguments.len() > 64
        || arguments
            .iter()
            .any(|arg| arg.is_empty() || arg.len() > 4096 || arg.contains('\0'))
    {
        return Err(TaskError(
            "commands require 1-64 nonempty bounded argv entries".into(),
        ));
    }
    Ok(())
}
