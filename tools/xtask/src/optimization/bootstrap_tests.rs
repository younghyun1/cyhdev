//! Synthetic setup and cleanup checks; never build optimized artifacts or start real fixtures.

use super::{
    bootstrap, bootstrap_inputs,
    bootstrap_resources::{Database, OwnedChild},
};
use std::{
    fs,
    path::Path,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(std::path::PathBuf);
impl Fixture {
    fn new() -> std::io::Result<Self> {
        let path = std::env::temp_dir().join(format!(
            "cyhdev-bootstrap-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path)?;
        Ok(Self(path))
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.0) {
            eprintln!("bootstrap test cleanup failed: {error}");
        }
    }
}

#[test]
fn default_inputs_never_reuse_previous_config_runtime_or_profile_names() -> crate::TaskResult<()> {
    let root = Path::new("/root with spaces");
    let first = bootstrap_inputs::inputs(root, "/bin/oha".into())?;
    let second = bootstrap_inputs::inputs(root, "/bin/oha".into())?;
    assert_ne!(first.name, second.name);
    assert_ne!(first.runtime, second.runtime);
    assert_ne!(first.directory, second.directory);
    assert!(
        first
            .directory
            .starts_with(root.join("target/optimization-inputs"))
    );
    assert_ne!(
        first.directory.join("config.json"),
        root.join("target/optimization-inputs/config.json")
    );
    assert!(first.runtime.starts_with("/tmp"));
    assert!(first.runtime.as_os_str().as_encoded_bytes().len() < 70);
    assert!(first.database.starts_with("cyhdev_optimization_") && first.database.len() <= 48);
    Ok(())
}

#[test]
fn cargo_artifact_receipt_finds_custom_build_directory_and_rejects_missing_binary()
-> crate::TaskResult<()> {
    let receipt = "{\"reason\":\"compiler-artifact\",\"target\":{\"name\":\"unrelated\"},\"executable\":\"/wrong\"}\n{\"reason\":\"compiler-artifact\",\"target\":{\"name\":\"optimization-fixtures\"},\"executable\":\"/custom build/fixture-tool\"}\n";
    assert_eq!(
        bootstrap::artifact(receipt)?,
        Path::new("/custom build/fixture-tool")
    );
    assert!(bootstrap::artifact("{}").is_err());
    assert!(bootstrap::artifact("invalid json").is_err());
    Ok(())
}

#[test]
fn managed_database_uses_loopback_tmpfs_and_no_reusable_volume()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let database = Database::new(&fixture.0, &fixture.0);
    let command = database.start_command("unique-owner", "sha256:synthetic");
    let args: Vec<_> = command.get_args().collect();
    for pair in [
        ["--publish", "127.0.0.1:35432:5432"],
        ["--tmpfs", "/var/lib/postgresql:rw,nosuid,size=1g"],
        ["--memory", "1g"],
        ["--name", "unique-owner"],
    ] {
        assert!(args.windows(2).any(|args| args == pair));
    }
    assert!(args.contains(&std::ffi::OsStr::new("--rm")));
    assert!(!args.contains(&std::ffi::OsStr::new("--volume")));
    Ok(())
}

#[test]
fn missing_or_untrusted_container_receipts_never_remove_an_existing_container()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let database = Database::new(&fixture.0, &fixture.0);
    database.cleanup()?;
    fs::write(
        fixture.0.join("postgres.cid"),
        "existing-production-container",
    )?;
    assert!(database.cleanup().is_err());
    fs::remove_file(fixture.0.join("postgres.cid"))?;
    #[cfg(unix)]
    {
        fs::write(fixture.0.join("outside"), "a".repeat(64))?;
        std::os::unix::fs::symlink(fixture.0.join("outside"), fixture.0.join("postgres.cid"))?;
        assert!(database.cleanup().is_err());
        fs::remove_file(fixture.0.join("postgres.cid"))?;
    }
    Ok(())
}

#[test]
fn prerequisite_environment_drops_application_and_pg_credentials() {
    let command = bootstrap_inputs::command("synthetic", Path::new("/workspace"));
    for key in [
        "DB_URL",
        "PGPASSWORD",
        "PGHOSTADDR",
        "PGSERVICE",
        "AWS_ACCESS_KEY_ID",
        "AWS_SECRET_ACCESS_KEY",
        "BASH_ENV",
        "ENV",
        "NODE_OPTIONS",
    ] {
        assert!(
            !command
                .get_envs()
                .any(|(name, value)| name == key && value.is_some())
        );
    }
}

#[test]
fn owned_child_failure_and_scope_drop_reap_only_the_spawned_process()
-> Result<(), Box<dyn std::error::Error>> {
    let mut failure = Command::new("sh");
    failure.args(["-c", "exit 17"]);
    assert!(OwnedChild::spawn(&mut failure)?.wait().is_err());
    let started = Instant::now();
    let mut sleeper = Command::new("sh");
    sleeper.args(["-c", "exec sleep 30"]);
    let mut child = OwnedChild::spawn(&mut sleeper)?;
    assert!(child.alive()?);
    drop(child);
    assert!(started.elapsed() < Duration::from_secs(5));
    Ok(())
}

#[test]
fn bootstrap_help_and_argument_errors_need_no_runtime_directory() {
    assert!(bootstrap::run(Path::new("/missing-checkout"), &["--help".into()]).is_ok());
    assert!(bootstrap::run(Path::new("/missing-checkout"), &["a".into(), "b".into()]).is_err());
}
