//! Optimized host-side artifact build through the deployment Docker toolchain.

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use crate::{TaskError, TaskResult, run_command};

const DEFAULT_APP_NAME: &str = "rust-be-template";
const DEFAULT_TARGET_TRIPLE: &str = "x86_64-unknown-linux-gnu";
const DEFAULT_TARGET_CPU: &str = "znver3";
const DEFAULT_DOCKER_PLATFORM: &str = "linux/amd64";
const DEFAULT_RUST_DOCKER_TAG: &str = "nightly";

struct ReleaseOptions {
    app_name: String,
    target_triple: String,
    target_cpu: String,
    docker_platform: String,
    rust_docker_tag: String,
    source_date_epoch: String,
}

pub(crate) fn run(root: &Path) -> TaskResult<()> {
    let options = ReleaseOptions::from_environment(root)?;
    let output_directory = artifact_directory(root, &options.target_triple);
    fs::create_dir_all(&output_directory).map_err(|error| {
        TaskError(format!(
            "failed to create optimized artifact directory {}: {error}",
            output_directory.display()
        ))
    })?;

    let mut command = Command::new("docker");
    command
        .args([
            "build",
            "--pull",
            "--file",
            "rust-be-template/Dockerfile",
            "--target",
            "artifact",
            "--platform",
        ])
        .arg(&options.docker_platform)
        .args(["--build-arg", &format!("APP_NAME={}", options.app_name)])
        .args([
            "--build-arg",
            &format!("RUST_TARGET={}", options.target_triple),
        ])
        .args([
            "--build-arg",
            &format!("RUST_TARGET_CPU={}", options.target_cpu),
        ])
        .args([
            "--build-arg",
            &format!("SOURCE_DATE_EPOCH={}", options.source_date_epoch),
        ])
        .args([
            "--build-arg",
            &format!("RUST_DOCKER_TAG={}", options.rust_docker_tag),
        ])
        .args(["--output"])
        .arg(format!("type=local,dest={}", output_directory.display()))
        .arg(".")
        .env("DOCKER_BUILDKIT", "1")
        .current_dir(root);
    run_command(&mut command)?;

    let artifact = output_directory.join(&options.app_name);
    validate_artifact(&artifact)?;
    println!("Optimized backend artifact: {}", artifact.display());
    Ok(())
}

impl ReleaseOptions {
    fn from_environment(root: &Path) -> TaskResult<Self> {
        let app_name = environment_value("APP_NAME", DEFAULT_APP_NAME)?;
        let target_triple = target_triple()?;
        let target_cpu = environment_value("TARGET_CPU", DEFAULT_TARGET_CPU)?;
        let docker_platform = environment_value("DOCKER_PLATFORM", DEFAULT_DOCKER_PLATFORM)?;
        let rust_docker_tag = environment_value("RUST_DOCKER_TAG", DEFAULT_RUST_DOCKER_TAG)?;
        let source_date_epoch = source_date_epoch(root)?;

        validate_token("APP_NAME", &app_name, 64, false)?;
        validate_token("TARGET_TRIPLE", &target_triple, 64, false)?;
        if target_triple != DEFAULT_TARGET_TRIPLE {
            return Err(TaskError(format!(
                "TARGET_TRIPLE must be {DEFAULT_TARGET_TRIPLE}; the compatibility builder provides GNU/Linux native dependencies only"
            )));
        }
        validate_token("TARGET_CPU", &target_cpu, 64, true)?;
        validate_token("DOCKER_PLATFORM", &docker_platform, 64, true)?;
        validate_token("RUST_DOCKER_TAG", &rust_docker_tag, 64, true)?;
        Ok(Self {
            app_name,
            target_triple,
            target_cpu,
            docker_platform,
            rust_docker_tag,
            source_date_epoch,
        })
    }
}

/// Returns an explicit source epoch or the current Git revision timestamp.
pub(crate) fn source_date_epoch(root: &Path) -> TaskResult<String> {
    match env::var("SOURCE_DATE_EPOCH") {
        Ok(value) => normalize_source_date_epoch(&value),
        Err(env::VarError::NotPresent) => git_commit_epoch(root),
        Err(env::VarError::NotUnicode(_)) => Err(TaskError(
            "SOURCE_DATE_EPOCH must contain valid UTF-8".to_owned(),
        )),
    }
}

fn git_commit_epoch(root: &Path) -> TaskResult<String> {
    let output = Command::new("git")
        .args(["show", "-s", "--format=%ct", "HEAD"])
        .current_dir(root)
        .output()
        .map_err(|error| TaskError(format!("failed to read the Git commit timestamp: {error}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(TaskError(format!(
            "git could not read the HEAD commit timestamp: {}",
            stderr.trim()
        )));
    }
    let value = String::from_utf8(output.stdout).map_err(|error| {
        TaskError(format!(
            "git returned a non-UTF-8 HEAD commit timestamp: {error}"
        ))
    })?;
    normalize_source_date_epoch(value.trim())
}

fn normalize_source_date_epoch(value: &str) -> TaskResult<String> {
    let epoch = value.parse::<u64>().map_err(|error| {
        TaskError(format!(
            "SOURCE_DATE_EPOCH must be an unsigned integer: {error}"
        ))
    })?;
    let epoch = i64::try_from(epoch).map_err(|error| {
        TaskError(format!(
            "SOURCE_DATE_EPOCH must fit in a signed 64-bit timestamp: {error}"
        ))
    })?;
    Ok(epoch.to_string())
}

fn target_triple() -> TaskResult<String> {
    match env::var("TARGET_TRIPLE") {
        Ok(value) => Ok(value),
        Err(env::VarError::NotPresent) => environment_value("RUST_TARGET", DEFAULT_TARGET_TRIPLE),
        Err(env::VarError::NotUnicode(_)) => Err(TaskError(
            "TARGET_TRIPLE must contain valid UTF-8".to_owned(),
        )),
    }
}

fn environment_value(key: &'static str, default: &'static str) -> TaskResult<String> {
    match env::var(key) {
        Ok(value) => Ok(value),
        Err(env::VarError::NotPresent) => Ok(default.to_owned()),
        Err(env::VarError::NotUnicode(_)) => {
            Err(TaskError(format!("{key} must contain valid UTF-8")))
        }
    }
}

fn validate_token(
    key: &'static str,
    value: &str,
    maximum_chars: usize,
    allow_slash_and_dot: bool,
) -> TaskResult<()> {
    let valid = !value.is_empty()
        && value.chars().count() <= maximum_chars
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(byte, b'-' | b'_')
                || (allow_slash_and_dot && matches!(byte, b'/' | b'.'))
        });
    if valid {
        Ok(())
    } else {
        Err(TaskError(format!(
            "{key} must be a nonempty ASCII build token of at most {maximum_chars} characters"
        )))
    }
}

fn artifact_directory(root: &Path, target_triple: &str) -> PathBuf {
    root.join("target").join(target_triple).join("release")
}

fn validate_artifact(path: &Path) -> TaskResult<()> {
    let metadata = fs::metadata(path).map_err(|error| {
        TaskError(format!(
            "optimized build did not produce {}: {error}",
            path.display()
        ))
    })?;
    if !metadata.is_file() || metadata.len() == 0 {
        return Err(TaskError(format!(
            "optimized artifact {} is empty or not a regular file",
            path.display()
        )));
    }
    #[cfg(unix)]
    if metadata.permissions().mode() & 0o111 == 0 {
        return Err(TaskError(format!(
            "optimized artifact {} is not executable",
            path.display()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{artifact_directory, normalize_source_date_epoch, validate_token};

    #[test]
    fn build_tokens_reject_shell_and_path_injection() {
        assert!(validate_token("APP_NAME", "rust-be-template", 64, false).is_ok());
        assert!(validate_token("APP_NAME", "../server", 64, false).is_err());
        assert!(validate_token("TARGET_CPU", "znver3", 64, true).is_ok());
        assert!(validate_token("TARGET_CPU", "znver3;touch", 64, true).is_err());
        assert!(validate_token("DOCKER_PLATFORM", "linux/amd64", 64, true).is_ok());
    }

    #[test]
    fn artifact_path_matches_cargo_release_layout() {
        assert_eq!(
            artifact_directory(Path::new("/repo"), "x86_64-unknown-linux-gnu"),
            Path::new("/repo/target/x86_64-unknown-linux-gnu/release")
        );
    }

    #[test]
    fn source_epoch_is_canonical_and_bounded_to_unsigned_values() -> crate::TaskResult<()> {
        assert_eq!(normalize_source_date_epoch("001234")?, "1234");
        assert!(normalize_source_date_epoch("-1").is_err());
        assert!(normalize_source_date_epoch("").is_err());
        assert!(normalize_source_date_epoch("tomorrow").is_err());
        assert!(normalize_source_date_epoch(&u64::MAX.to_string()).is_err());
        Ok(())
    }
}
