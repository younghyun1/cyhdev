//! Best-effort hardware capture for interpreting machine-dependent results.

use std::{
    env,
    ffi::OsString,
    fs,
    path::Path,
    process::Command,
    thread,
};

use serde::{Deserialize, Serialize};

use crate::{
    config::fnv1a64_hex,
    error::{HarnessError, HarnessResult},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedHardware {
    pub operating_system: String,
    pub kernel_version: Option<String>,
    pub architecture: String,
    pub logical_cpus: Option<usize>,
    pub cpu_model: Option<String>,
    pub memory_bytes: Option<u64>,
    pub rustc_version: Option<String>,
}

pub fn observe() -> ObservedHardware {
    let logical_cpus = match thread::available_parallelism() {
        Ok(count) => Some(count.get()),
        Err(_source) => None,
    };
    ObservedHardware {
        operating_system: std::env::consts::OS.to_owned(),
        kernel_version: command_output(OsString::from("uname"), &["-r"]),
        architecture: std::env::consts::ARCH.to_owned(),
        logical_cpus,
        cpu_model: cpu_model(),
        memory_bytes: memory_bytes(),
        rustc_version: rustc_version(),
    }
}

/// Hashes the declared environment together with machine and compiler facts.
pub fn environment_digest(
    declared_digest: &str,
    hardware: &ObservedHardware,
    compiled_profile: &str,
    implementation_digest: &str,
    executor_kind: &str,
    target: &str,
    resolved_address: Option<&str>,
) -> HarnessResult<String> {
    let bytes = serde_json::to_vec(&(
        declared_digest,
        hardware,
        compiled_profile,
        implementation_digest,
        executor_kind,
        target,
        resolved_address,
    )).map_err(|source| HarnessError::Json {
        path: Path::new("<observed-environment>").to_path_buf(),
        source,
    })?;
    Ok(fnv1a64_hex(&bytes))
}

fn cpu_model() -> Option<String> {
    match std::env::consts::OS {
        "linux" => read_linux_cpu_model(),
        "macos" => read_sysctl("machdep.cpu.brand_string"),
        _ => None,
    }
}

fn memory_bytes() -> Option<u64> {
    match std::env::consts::OS {
        "linux" => read_linux_memory_bytes(),
        "macos" => read_sysctl("hw.memsize").and_then(|value| value.parse::<u64>().ok()),
        _ => None,
    }
}

fn read_linux_cpu_model() -> Option<String> {
    let cpuinfo = fs::read_to_string("/proc/cpuinfo").ok()?;
    cpuinfo.lines().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        if matches!(key.trim(), "model name" | "Hardware") {
            non_empty(value)
        } else {
            None
        }
    })
}

fn read_linux_memory_bytes() -> Option<u64> {
    let memory = fs::read_to_string("/proc/meminfo").ok()?;
    let line = memory.lines().find(|line| line.starts_with("MemTotal:"))?;
    let kibibytes = line
        .split_ascii_whitespace()
        .nth(1)?
        .parse::<u64>()
        .ok()?;
    kibibytes.checked_mul(1_024)
}

fn read_sysctl(name: &str) -> Option<String> {
    let output = Command::new("sysctl").args(["-n", name]).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8(output.stdout).ok()?;
    non_empty(&value)
}

fn rustc_version() -> Option<String> {
    let rustc = match env::var_os("RUSTC") {
        Some(path) => path,
        None => OsString::from("rustc"),
    };
    command_output(rustc, &["-vV"])
}

fn command_output(program: OsString, arguments: &[&str]) -> Option<String> {
    let output = Command::new(program).args(arguments).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8(output.stdout).ok()?;
    non_empty(&value)
}

fn non_empty(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::{ObservedHardware, environment_digest};

    #[test]
    fn observed_digest_changes_with_toolchain() -> Result<(), String> {
        let mut hardware = ObservedHardware {
            operating_system: "linux".to_owned(),
            kernel_version: Some("6.0.0".to_owned()),
            architecture: "x86_64".to_owned(),
            logical_cpus: Some(8),
            cpu_model: Some("fixture".to_owned()),
            memory_bytes: Some(16 * 1024 * 1024 * 1024),
            rustc_version: Some("rustc nightly-a".to_owned()),
        };
        let first = environment_digest(
            "fnv1a64:0000000000000000", &hardware, "debug",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "http", "http://127.0.0.1:3000", Some("127.0.0.1:3000"),
        )
            .map_err(|error| error.to_string())?;
        hardware.rustc_version = Some("rustc nightly-b".to_owned());
        let second = environment_digest(
            "fnv1a64:0000000000000000", &hardware, "debug",
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "http", "http://127.0.0.1:3000", Some("127.0.0.1:3000"),
        )
            .map_err(|error| error.to_string())?;

        if first == second {
            Err("toolchain change did not alter the observed digest".to_owned())
        } else {
            Ok(())
        }
    }
}
