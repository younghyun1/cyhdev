//! Deferred repository-wide verification commands.

use std::{env, path::Path, process::Command};

use crate::{TaskError, TaskResult, run_command, run_native_clippy, run_package, run_wasm_clippy};

type ReviewStep = (&'static str, fn(&Path) -> TaskResult<()>);

pub(crate) fn run_format_check(root: &Path) -> TaskResult<()> {
    run_command(
        Command::new("cargo")
            .args([
                "fmt",
                "--package",
                "rust-be-template",
                "--package",
                "xtask",
                "--",
                "--check",
            ])
            .current_dir(root),
    )
}

pub(crate) fn run_unit_tests(root: &Path) -> TaskResult<()> {
    run_command(
        Command::new("cargo")
            .args([
                "test",
                "--locked",
                "--workspace",
                "--exclude",
                "block_breaker",
                "--exclude",
                "ray_tracer",
                "--all-features",
                "--no-fail-fast",
            ])
            .current_dir(root),
    )
}

pub(crate) fn run_database_integration(root: &Path) -> TaskResult<()> {
    require_test_database_url()?;
    run_command(
        Command::new("cargo")
            .args([
                "test",
                "--locked",
                "--package",
                "rust-be-template",
                "--test",
                "postgres_account_boundaries",
                "--test",
                "postgres_account_identity",
                "--test",
                "postgres_account_http_boundaries",
                "--test",
                "postgres_account_oidc",
                "--test",
                "postgres_account_lifecycle",
                "--test",
                "postgres_content_write_linearization",
                "--test",
                "postgres_cache_consistency",
                "--test",
                "postgres_profile_picture_history",
                "--test",
                "postgres_forum",
                "--test",
                "postgres_photography_invariants",
                "--test",
                "postgres_wasm",
                "--test",
                "postgres_retention_notifications",
                "--test",
                "postgres_authorization_admin",
                "--no-fail-fast",
                "--",
                "--ignored",
                "--skip",
                "embedded_migration_chain_reverts_and_reapplies",
                "--skip",
                "account_lifecycle_migration_reverts_and_reapplies",
            ])
            .current_dir(root),
    )
}

pub(crate) fn run_migration_rollback(root: &Path) -> TaskResult<()> {
    require_test_database_url()?;
    run_command(
        Command::new("cargo")
            .args([
                "test",
                "--locked",
                "--package",
                "rust-be-template",
                "--test",
                "postgres_account_boundaries",
                "embedded_migration_chain_reverts_and_reapplies",
                "--",
                "--ignored",
                "--exact",
                "--test-threads=1",
            ])
            .current_dir(root),
    )?;
    run_command(
        Command::new("cargo")
            .args([
                "test",
                "--locked",
                "--package",
                "rust-be-template",
                "--test",
                "postgres_account_lifecycle",
                "account_lifecycle_migration_reverts_and_reapplies",
                "--",
                "--ignored",
                "--exact",
                "--test-threads=1",
            ])
            .current_dir(root),
    )?;
    run_command(
        Command::new("cargo")
            .args([
                "test",
                "--locked",
                "--package",
                "rust-be-template",
                "--test",
                "postgres_migration_guards",
                "--",
                "--ignored",
                "--test-threads=1",
            ])
            .current_dir(root),
    )
}

pub(crate) fn run_openapi_drift_check(root: &Path) -> TaskResult<()> {
    run_command(
        Command::new("cargo")
            .args([
                "run",
                "--locked",
                "--package",
                "rust-be-template",
                "--bin",
                "openapi-contracts",
                "--",
                "check",
            ])
            .current_dir(root),
    )
}

pub(crate) fn run_frontend_checks(root: &Path) -> TaskResult<()> {
    let frontend = root.join("solid-csr-spa-template");
    run_command(
        Command::new("npm")
            .args(["ci", "--no-audit", "--no-fund"])
            .current_dir(&frontend),
    )?;
    let checks: [(&str, &[&str]); 4] = [
        ("typecheck", &["run", "typecheck"]),
        ("lint", &["run", "lint", "--", "--max-warnings", "0"]),
        ("unit tests", &["run", "test"]),
        ("build", &["run", "build"]),
    ];
    let mut failures = Vec::new();
    for (name, arguments) in checks {
        let result = run_command(Command::new("npm").args(arguments).current_dir(&frontend));
        if let Err(error) = result {
            failures.push(format!("{name}: {error}"));
        }
    }
    finish_operation("frontend checks", failures)
}

pub(crate) fn run_image_smoke(root: &Path) -> TaskResult<()> {
    // The smoke target compiles current sources in a non-release profile and
    // validates the runtime image layout without requiring deployment secrets.
    run_command(
        Command::new("docker")
            .args([
                "build",
                "--pull",
                "--target",
                "smoke",
                "--file",
                "rust-be-template/Dockerfile",
                "--tag",
                "cyhdev-backend:smoke",
                ".",
            ])
            .current_dir(root),
    )
}

pub(crate) fn run_secret_scan(root: &Path) -> TaskResult<()> {
    crate::secret_scan::run(root)
}

pub(crate) fn run_final_review(root: &Path) -> TaskResult<()> {
    let steps: [ReviewStep; 11] = [
        ("format", run_format_check),
        ("Clippy", run_clippy),
        ("unit tests", run_unit_tests),
        ("OpenAPI drift", run_openapi_drift_check),
        ("frontend checks", run_frontend_checks),
        ("database integration", run_database_integration),
        ("migration rollback", run_migration_rollback),
        ("image smoke", run_image_smoke),
        ("throughput thresholds", run_throughput_thresholds),
        ("secret scan", run_secret_scan),
        ("evidence manifest", crate::evidence_manifest::run),
    ];
    let mut failures = Vec::new();
    for (name, step) in steps {
        println!("==> {name}");
        if let Err(error) = step(root) {
            failures.push(format!("{name}: {error}"));
        }
    }
    finish_operation("final review", failures)
}

fn run_clippy(root: &Path) -> TaskResult<()> {
    run_native_clippy(root)?;
    run_wasm_clippy(root)
}

fn run_throughput_thresholds(root: &Path) -> TaskResult<()> {
    // Keep the portable fixture as a harness regression, then require the
    // hardware-specific HTTP baseline for an actual backend capacity claim.
    let mut failures = Vec::new();
    if let Err(error) = run_package(root, "throughput-harness", &[]) {
        failures.push(format!("fixture: {error}"));
    }
    if let Err(error) = run_http_throughput_thresholds(root) {
        failures.push(format!("HTTP: {error}"));
    }
    finish_operation("throughput thresholds", failures)
}

fn run_http_throughput_thresholds(root: &Path) -> TaskResult<()> {
    let target = required_environment_value("THROUGHPUT_HTTP_TARGET")?;
    let environment = required_environment_value("THROUGHPUT_HTTP_ENVIRONMENT")?;
    let thresholds = required_environment_value("THROUGHPUT_HTTP_THRESHOLDS")?;
    let output = match env::var("THROUGHPUT_HTTP_OUTPUT") {
        Ok(value) if !value.trim().is_empty() => value,
        Ok(_) | Err(env::VarError::NotPresent) => "target/throughput/backend-http.json".to_owned(),
        Err(env::VarError::NotUnicode(_)) => {
            return Err(TaskError(
                "THROUGHPUT_HTTP_OUTPUT must contain valid UTF-8".to_owned(),
            ));
        }
    };
    run_package(
        root,
        "throughput-harness",
        &[
            "run".to_owned(),
            "--target".to_owned(),
            target,
            "--environment".to_owned(),
            environment,
            "--thresholds".to_owned(),
            thresholds,
            "--output".to_owned(),
            output,
        ],
    )
}

fn required_environment_value(key: &'static str) -> TaskResult<String> {
    match env::var(key) {
        Ok(value) if !value.trim().is_empty() => Ok(value),
        Ok(_) | Err(env::VarError::NotPresent) => Err(TaskError(format!(
            "{key} is required for the hardware-specific backend throughput gate"
        ))),
        Err(env::VarError::NotUnicode(_)) => {
            Err(TaskError(format!("{key} must contain valid UTF-8")))
        }
    }
}

fn require_test_database_url() -> TaskResult<()> {
    crate::test_database::validate()
}

fn finish_operation(operation: &str, failures: Vec<String>) -> TaskResult<()> {
    if failures.is_empty() {
        Ok(())
    } else {
        Err(TaskError(format!(
            "{operation} failed:\n  {}",
            failures.join("\n  ")
        )))
    }
}
