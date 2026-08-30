//! Root-relative development and verification commands for the workspace.

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
        "backend" => run_package(&root, "rust-be-template", &forwarded),
        "clippy" => {
            run_native_clippy(&root)?;
            run_wasm_clippy(&root)
        }
        "frontend" => run_command(
            Command::new("npm")
                .args(["run", "dev"])
                .current_dir(root.join("solid-csr-spa-template")),
        ),
        "frontend-build" => {
            let frontend = root.join("solid-csr-spa-template");
            run_command(Command::new("npm").args(["ci"]).current_dir(&frontend))?;
            run_command(
                Command::new("npm")
                    .args(["run", "build"])
                    .current_dir(frontend),
            )
        }
        "image" => run_command(
            Command::new("docker")
                .args([
                    "build",
                    "--file",
                    "rust-be-template/Dockerfile",
                    "--tag",
                    "cyhdev-backend:dev",
                    ".",
                ])
                .current_dir(&root),
        ),
        "test" => run_command(
            Command::new("cargo")
                .args(["test", "--workspace", "--all-features"])
                .current_dir(&root),
        ),
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
    command.args(["run", "--package", package]);
    if !arguments.is_empty() {
        command.arg("--").args(arguments);
    }
    command.current_dir(root);
    run_command(&mut command)
}

fn run_native_clippy(root: &Path) -> TaskResult<()> {
    run_command(
        Command::new("cargo")
            .args([
                "clippy",
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

fn run_wasm_clippy(root: &Path) -> TaskResult<()> {
    run_command(
        Command::new("cargo")
            .args([
                "clippy",
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
        "Commands:\n  backend         Run the backend\n  clippy          Run the implementation stage gate\n  frontend        Run the frontend development server\n  frontend-build  Install locked frontend dependencies and build assets\n  image           Build the local development image\n  test            Run Rust tests during the final review\n  wasm-clippy     Check the WebAssembly packages for their target"
    );
}
