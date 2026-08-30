//! Root-relative development and verification commands for the workspace.

mod evidence_manifest;
mod review;
mod secret_scan;
mod test_database;

use std::{
    env,
    error::Error,
    fmt::{self, Display, Formatter},
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

type TaskResult<T> = Result<T, TaskError>;

#[derive(Debug)]
struct TaskError(String);

impl Display for TaskError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for TaskError {}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("xtask: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> TaskResult<()> {
    let root = workspace_root()?;
    let mut arguments = env::args().skip(1);
    let command = match arguments.next() {
        Some(command) => command,
        None => {
            print_help();
            return Ok(());
        }
    };
    let forwarded: Vec<String> = arguments.collect();

    match command.as_str() {
        "backend" => run_backend(&root, &forwarded),
        "build" => run_native_build(&root),
        "clippy" => {
            run_native_clippy(&root)?;
            run_wasm_clippy(&root)
        }
        "frontend" => run_command(
            Command::new("npm")
                .args(["run", "dev"])
                .current_dir(root.join("solid-csr-spa-template")),
        ),
        "db-integration" => review::run_database_integration(&root),
        "evidence" => evidence_manifest::run(&root),
        "final-review" => review::run_final_review(&root),
        "fmt" => review::run_format_check(&root),
        "frontend-build" => {
            let frontend = root.join("solid-csr-spa-template");
            run_command(Command::new("npm").args(["ci"]).current_dir(&frontend))?;
            run_command(
                Command::new("npm")
                    .args(["run", "build"])
                    .current_dir(frontend),
            )
        }
        "frontend-check" => review::run_frontend_checks(&root),
        "image" => run_image(&root),
        "image-smoke" => review::run_image_smoke(&root),
        "migration-rollback" => review::run_migration_rollback(&root),
        "openapi" => review::run_openapi_drift_check(&root),
        "secret-scan" => review::run_secret_scan(&root),
        "test" | "unit" => review::run_unit_tests(&root),
        "throughput" => run_package(&root, "throughput-harness", &forwarded),
        "wasm-build" => run_wasm_build(&root),
        "wasm-clippy" => run_wasm_clippy(&root),
        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        unknown => Err(TaskError(format!("unknown command `{unknown}`"))),
    }
}

fn workspace_root() -> TaskResult<PathBuf> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    match manifest_dir.parent().and_then(Path::parent) {
        Some(root) => Ok(root.to_path_buf()),
        None => Err(TaskError(
            "could not resolve the workspace root from the xtask manifest".to_owned(),
        )),
    }
}

fn run_command(command: &mut Command) -> TaskResult<()> {
    let rendered = format!("{command:?}");
    let status = command
        .status()
        .map_err(|error| TaskError(format!("failed to start {rendered}: {error}")))?;
    if status.success() {
        Ok(())
    } else {
        Err(TaskError(format!(
            "{rendered} exited with status {status}"
        )))
    }
}

fn run_package(root: &Path, package: &str, arguments: &[String]) -> TaskResult<()> {
    let mut command = Command::new("cargo");
    command.args(["run", "--locked", "--package", package]);
    if !arguments.is_empty() {
        command.arg("--").args(arguments);
    }
    command.current_dir(root);
    run_command(&mut command)
}

fn run_backend(root: &Path, arguments: &[String]) -> TaskResult<()> {
    let backend = root.join("rust-be-template");
    let mut command = Command::new("cargo");
    command.args(["run", "--locked", "--package", "rust-be-template"]);
    if !arguments.is_empty() {
        command.arg("--").args(arguments);
    }
    // Runtime paths such as .env, Geo-IP bundles, certificates, and the search
    // index are intentionally backend-relative.
    command.current_dir(backend);
    run_command(&mut command)
}

fn run_native_build(root: &Path) -> TaskResult<()> {
    run_command(
        Command::new("cargo")
            .args([
                "build",
                "--locked",
                "--workspace",
                "--exclude",
                "block_breaker",
                "--exclude",
                "ray_tracer",
            ])
            .current_dir(root),
    )
}

fn run_image(root: &Path) -> TaskResult<()> {
    let source_date_epoch = match env::var("SOURCE_DATE_EPOCH") {
        Ok(value) => value,
        Err(env::VarError::NotPresent) => "0".to_owned(),
        Err(env::VarError::NotUnicode(_)) => {
            return Err(TaskError(
                "SOURCE_DATE_EPOCH must contain valid UTF-8".to_owned(),
            ));
        }
    };
    let mut command = Command::new("docker");
    command
        .args([
            "build",
            "--pull",
            "--file",
            "rust-be-template/Dockerfile",
            "--tag",
            "cyhdev-backend:dev",
            "--build-arg",
        ])
        .arg(format!("SOURCE_DATE_EPOCH={source_date_epoch}"))
        .arg(".")
        .current_dir(root);
    run_command(&mut command)
}

fn run_native_clippy(root: &Path) -> TaskResult<()> {
    run_command(
        Command::new("cargo")
            .args([
                "clippy",
                "--locked",
                "--workspace",
                "--exclude",
                "block_breaker",
                "--exclude",
                "ray_tracer",
                "--all-targets",
                "--all-features",
                "--",
                "-D",
                "warnings",
            ])
            .current_dir(root),
    )
}

fn run_wasm_build(root: &Path) -> TaskResult<()> {
    for package in ["wasm_demos/block_breaker", "wasm_demos/ray_tracer"] {
        run_command(
            Command::new("wasm-pack")
                .args([
                    "build",
                    package,
                    "--target",
                    "web",
                    "--profile",
                    "wasm-release",
                    "--",
                    "--locked",
                ])
                .env("RUSTFLAGS", "--cfg=web_sys_unstable_apis")
                .current_dir(root),
        )?;
    }
    Ok(())
}

fn run_wasm_clippy(root: &Path) -> TaskResult<()> {
    run_command(
        Command::new("cargo")
            .args([
                "clippy",
                "--locked",
                "--package",
                "block_breaker",
                "--package",
                "ray_tracer",
                "--target",
                "wasm32-unknown-unknown",
                "--all-targets",
                "--all-features",
                "--",
                "-D",
                "warnings",
            ])
            .env("RUSTFLAGS", "--cfg=web_sys_unstable_apis")
            .current_dir(root),
    )
}

fn print_help() {
    println!(
        "Commands:\n  backend             Run the backend\n  build               Build native workspace packages from locked inputs\n  clippy              Run the implementation stage gate\n  db-integration      Run ignored PostgreSQL integration cases\n  evidence            Validate W3/W8 registrations and evidence\n  final-review        Run every deferred review gate and aggregate failures\n  fmt                 Check Rust formatting\n  frontend            Run the frontend development server\n  frontend-build      Install locked frontend dependencies and build assets\n  frontend-check      Run frontend type, lint, and unit checks\n  image               Build the local development image from locked inputs\n  image-smoke         Build the non-release Docker smoke target\n  migration-rollback  Revert and reapply every embedded migration\n  openapi              Check generated frontend contracts for drift\n  secret-scan          Scan the current tree and all Git refs with redacted reports\n  test, unit           Run native unit and non-database tests\n  throughput          Replay the recorded workload and enforce thresholds\n  wasm-build          Build WebAssembly packages from locked inputs\n  wasm-clippy         Check the WebAssembly packages for their target"
    );
}
