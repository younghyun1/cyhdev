//! Exercise Cargo argv encoding and profile flushing without inherited environment.

use std::{fs, path::PathBuf, process::Command};

use super::{artifacts, container};

struct Fixture(PathBuf);

impl Drop for Fixture {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.0) {
            eprintln!("profile fixture cleanup failed: {error}");
        }
    }
}

/// The seed supervisor clears environment variables. The compiled fallback
/// must still flush into the native run, including checkout paths with spaces.
#[test]
fn environment_cleared_child_flushes_to_native_path_with_spaces()
-> Result<(), Box<dyn std::error::Error>> {
    let directory =
        std::env::temp_dir().join(format!("cyhdev-profile-probe-{}", std::process::id()));
    fs::create_dir(&directory)?;
    let fixture = Fixture(directory);
    let run = fixture.0.join("native run");
    fs::create_dir_all(run.join("raw/pgo"))?;
    let project = fixture.0.join("project");
    fs::create_dir_all(project.join("src"))?;
    fs::write(
        project.join("Cargo.toml"),
        "[package]\nname = \"profile-probe\"\nversion = \"0.1.0\"\nedition = \"2024\"\n[workspace]\n",
    )?;
    fs::write(
        project.join("src/main.rs"),
        "fn main() { println!(\"{}\", std::env::args().count()); }\n",
    )?;
    crate::run_command(
        Command::new("cargo")
            .args(["generate-lockfile", "--offline"])
            .current_dir(&project),
    )?;
    crate::run_command(
        Command::new("cargo")
            .args([
                "build",
                "--offline",
                "--locked",
                "--target",
                container::TARGET,
            ])
            .env(
                "CARGO_ENCODED_RUSTFLAGS",
                container::encoded_flags(&run, "instrumented", "x86-64")?,
            )
            .current_dir(&project),
    )?;
    let output = Command::new(
        project
            .join("target")
            .join(container::TARGET)
            .join("debug/profile-probe"),
    )
    .env_clear()
    .output()?;
    assert!(output.status.success());
    assert_eq!(output.stdout, b"1\n");
    let profiles = artifacts::profiles(&run, "pgo")?;
    assert_eq!(profiles.len(), 1);
    assert!(
        profiles[0]
            .rsplit('/')
            .next()
            .is_some_and(|name| name.starts_with("default_"))
    );
    Ok(())
}

/// A unit separator would create another compiler flag rather than a path.
#[test]
fn profile_path_cannot_inject_an_encoded_argument() {
    assert!(
        container::encoded_flags(
            std::path::Path::new("/tmp/a\u{1f}-Cpanic=abort"),
            "instrumented",
            "x86-64"
        )
        .is_err()
    );
}
