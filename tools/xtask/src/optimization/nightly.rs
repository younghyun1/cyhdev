//! Resolve the current nightly once per campaign without discarding stable caches.

use std::{env, process::Command};

use crate::{TaskError, TaskResult};

use super::process;

const MANIFEST: &str = "https://static.rust-lang.org/dist/channel-rust-nightly.toml";
const MAX_MANIFEST_BYTES: usize = 2 * 1024 * 1024;

/// A dated toolchain is a stable cache input and prevents mid-campaign updates.
pub(super) fn requested() -> TaskResult<Option<String>> {
    match env::var("CYHDEV_OPT_LATEST_NIGHTLY") {
        Err(env::VarError::NotPresent) => return Ok(None),
        Ok(value) if value == "1" => {}
        _ => {
            return Err(TaskError(
                "CYHDEV_OPT_LATEST_NIGHTLY must be 1 or unset".into(),
            ));
        }
    }
    let manifest = match process::output(&mut manifest_command()) {
        Ok(manifest) => manifest,
        Err(error) => {
            return Err(TaskError(format!(
                "cannot fetch the latest nightly manifest: {error}"
            )));
        }
    };
    let toolchain = from_manifest(&manifest)?;
    println!("Optimization compiler: {toolchain}");
    Ok(Some(toolchain))
}

/// Bound the lookup and bypass intermediary freshness caches on every invocation.
pub(super) fn manifest_command() -> Command {
    let mut command = Command::new("curl");
    command.args([
        "--fail",
        "--silent",
        "--show-error",
        "--location",
        "--proto",
        "=https",
        "--proto-redir",
        "=https",
        "--max-time",
        "30",
        "--max-filesize",
        "2097152",
        "--header",
        "Cache-Control: no-cache",
        MANIFEST,
    ]);
    command
}

/// Only read the top-level date; package tables contain unrelated version dates.
pub(super) fn from_manifest(manifest: &str) -> TaskResult<String> {
    if manifest.len() > MAX_MANIFEST_BYTES {
        return Err(TaskError("nightly manifest exceeds 2 MiB".into()));
    }
    let mut date = None;
    let mut version = false;
    for line in manifest
        .lines()
        .map(str::trim)
        .take_while(|line| !line.starts_with('['))
    {
        if line == "manifest-version = \"2\"" {
            version = true;
        } else if let Some(value) = line
            .strip_prefix("date = \"")
            .and_then(|value| value.strip_suffix('"'))
        {
            if date.is_some() || !valid_date(value) {
                return Err(TaskError(
                    "nightly manifest has an invalid or duplicate date".into(),
                ));
            }
            date = Some(value);
        }
    }
    match (version, date) {
        (true, Some(date)) => Ok(format!("nightly-{date}")),
        _ => Err(TaskError(
            "nightly manifest requires version 2 and a top-level date".into(),
        )),
    }
}

/// Reject malformed dates before they become Docker or rustup arguments.
fn valid_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes
            .iter()
            .enumerate()
            .any(|(index, byte)| !matches!(index, 4 | 7) && !byte.is_ascii_digit())
    {
        return false;
    }
    let (year, month, day) = match (
        value[..4].parse::<u32>(),
        value[5..7].parse::<u32>(),
        value[8..].parse::<u32>(),
    ) {
        (Ok(year), Ok(month), Ok(day)) => (year, month, day),
        _ => return false,
    };
    let maximum = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year.is_multiple_of(400) || (year.is_multiple_of(4) && !year.is_multiple_of(100)) => {
            29
        }
        2 => 28,
        _ => return false,
    };
    year >= 2015 && day >= 1 && day <= maximum
}
