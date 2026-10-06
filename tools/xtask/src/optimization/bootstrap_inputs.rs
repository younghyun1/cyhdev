//! Fresh campaign identities, host prerequisites, and a restricted tool environment.

use super::{config, process};
use crate::{TaskError, TaskResult};
use std::{
    env, fs,
    net::TcpListener,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

/// Preserve tool configuration and measurement controls, never application credentials.
pub(super) fn command(program: impl AsRef<std::ffi::OsStr>, root: &Path) -> Command {
    let mut command = Command::new(program);
    command.env_clear().current_dir(root);
    for key in [
        "PATH",
        "HOME",
        "USER",
        "LOGNAME",
        "LANG",
        "LC_ALL",
        "CARGO_HOME",
        "RUSTUP_HOME",
        "RUSTUP_TOOLCHAIN",
        "DOCKER_HOST",
        "DOCKER_CONTEXT",
        "DOCKER_CONFIG",
        "DOCKER_TLS_VERIFY",
        "DOCKER_CERT_PATH",
        "XDG_CACHE_HOME",
        "PLAYWRIGHT_BROWSERS_PATH",
        "CYHDEV_OPT_SERVER_CPU_SET",
        "TOKIO_WORKER_THREADS",
    ] {
        if let Some(value) = env::var_os(key) {
            command.env(key, value);
        }
    }
    command
}

pub(super) struct Inputs {
    pub name: String,
    pub runtime: PathBuf,
    pub directory: PathBuf,
    pub cpu: String,
    pub database: String,
    pub oha: PathBuf,
}

/// Check dependencies before creating runtime state or starting any fixture.
pub(super) fn prerequisites(root: &Path) -> TaskResult<PathBuf> {
    let endpoint = process::output(command("docker", root).args(["context", "inspect", "--format", "{{.Endpoints.docker.Host}}"])).map_err(|error| TaskError(format!("Docker is unavailable; start the local Docker daemon and rerun ./build_pgo_and_bolt.sh: {error}")))?;
    if !endpoint.starts_with("unix://") {
        return Err(TaskError("managed fixtures require a local Docker Unix socket; use an explicit config for a separately prepared runner".into()));
    }
    process::output(command("docker", root).args(["info", "--format", "{{.OSType}}"])).map_err(
        |error| {
            TaskError(format!(
                "Docker daemon is unavailable; start it and rerun ./build_pgo_and_bolt.sh: {error}"
            ))
        },
    )?;
    for program in [
        "pg_dump",
        "createdb",
        "dropdb",
        "pg_isready",
        "openssl",
        "npm",
        "node",
        "curl",
    ] {
        executable(program)?;
    }
    let version = process::output(command("pg_dump", root).arg("--version"))?;
    if !version.contains("(PostgreSQL) 18.") {
        return Err(TaskError(
            "managed fixtures require PostgreSQL 18 client tools to match postgres:18".into(),
        ));
    }
    for name in ["new_bundle_ipv4.db", "new_bundle_ipv6.db"] {
        if !root.join("rust-be-template").join(name).is_file() {
            return Err(TaskError(format!(
                "optimization prerequisite is missing: public Geo-IP bundle {name}"
            )));
        }
    }
    for port in [35432, 34901, 3465, 18555, 18443, 18444] {
        if TcpListener::bind(("127.0.0.1", port)).is_err() {
            return Err(TaskError(format!(
                "fixture port 127.0.0.1:{port} is occupied; the build will not stop or reuse its listener"
            )));
        }
    }
    match env::var_os("CYHDEV_OHA") {
        Some(path) => {
            let path = PathBuf::from(path);
            if !path.is_absolute() || !is_executable(&path) {
                return Err(TaskError(
                    "CYHDEV_OHA must name an absolute executable".into(),
                ));
            }
            Ok(path)
        }
        None => executable("oha"),
    }
}

fn is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        matches!(fs::metadata(path), Ok(metadata) if metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
    }
    #[cfg(not(unix))]
    {
        path.is_file()
    }
}

fn executable(name: &str) -> TaskResult<PathBuf> {
    if let Some(paths) = env::var_os("PATH") {
        for directory in env::split_paths(&paths) {
            let path = directory.join(name);
            if is_executable(&path) {
                return path
                    .canonicalize()
                    .map_err(|error| TaskError(format!("cannot resolve {name}: {error}")));
            }
        }
    }
    Err(TaskError(format!(
        "optimization prerequisite is missing: {name}; install it and rerun ./build_pgo_and_bolt.sh"
    )))
}

/// Every default build gets new inputs; old generated configs are never consulted.
pub(super) fn inputs(root: &Path, oha: PathBuf) -> TaskResult<Inputs> {
    let nanos = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(time) => time.as_nanos(),
        Err(error) => return Err(TaskError(format!("cannot name a fresh campaign: {error}"))),
    };
    let id = format!("{nanos:x}_{:x}", std::process::id());
    let name = format!("cyhdev-pgo-bolt-{id}");
    let runtime = PathBuf::from(format!("/tmp/cyh-opt-{id}"));
    let directory = root.join("target/optimization-inputs").join(&name);
    let database = format!("cyhdev_optimization_{id}");
    if database.len() > 48 {
        return Err(TaskError(
            "generated fixture database name exceeds its bound".into(),
        ));
    }
    let cpu = match env::var("TARGET_CPU") {
        Ok(cpu) => cpu,
        Err(env::VarError::NotPresent) => "znver3".into(),
        Err(error) => return Err(TaskError(format!("invalid TARGET_CPU: {error}"))),
    };
    config::token(&cpu)?;
    Ok(Inputs {
        name,
        runtime,
        directory,
        cpu,
        database,
        oha,
    })
}
