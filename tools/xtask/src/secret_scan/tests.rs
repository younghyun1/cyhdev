//! Filesystem fixtures exercise index modes without contacting submodule remotes.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

use super::source_inventory::copy_public_source;

type TestResult = Result<(), Box<dyn std::error::Error>>;
static NEXT_DIRECTORY: AtomicUsize = AtomicUsize::new(0);

struct Fixture {
    directory: PathBuf,
    root: PathBuf,
    snapshot: PathBuf,
}

impl Fixture {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let id = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let directory =
            std::env::temp_dir().join(format!("cyhdev-secret-test-{}-{id}", std::process::id()));
        fs::create_dir(&directory)?;
        let fixture = Self {
            root: directory.join("source"),
            snapshot: directory.join("snapshot"),
            directory,
        };
        fs::create_dir(&fixture.root)?;
        fs::create_dir(&fixture.snapshot)?;
        git(&fixture.root, &["init", "--quiet"])?;
        fs::write(fixture.root.join("README.md"), "public fixture")?;
        git(&fixture.root, &["add", "README.md"])?;
        git(
            &fixture.root,
            &[
                "-c",
                "user.name=Fixture",
                "-c",
                "user.email=fixture@example.invalid",
                "commit",
                "--quiet",
                "-m",
                "fixture",
            ],
        )?;
        Ok(fixture)
    }

    fn submodule(&self, name: &str, initialized: bool) -> TestResult {
        let child = self.root.join(name);
        fs::create_dir_all(&child)?;
        let revision = git(&self.root, &["rev-parse", "HEAD"])?;
        git(
            &self.root,
            &[
                "update-index",
                "--add",
                "--cacheinfo",
                &format!("160000,{},{}", revision.trim(), name),
            ],
        )?;
        if initialized {
            git(&child, &["init", "--quiet"])?;
            fs::write(child.join("source.rs"), "// public source\n")?;
            fs::write(child.join(".gitignore"), ".env\n")?;
            fs::write(
                child.join(".env"),
                "PRIVATE_FIXTURE_MUST_NOT_ENTER_SNAPSHOT",
            )?;
            git(&child, &["add", "source.rs", ".gitignore"])?;
        }
        Ok(())
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _cleanup_result = fs::remove_dir_all(&self.directory);
    }
}

fn git(root: &Path, arguments: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(root)
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "fixture Git command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(String::from_utf8(output.stdout)?)
}

#[test]
fn initialized_submodules_include_public_source_and_history_roots() -> TestResult {
    let fixture = Fixture::new()?;
    fixture.submodule("vendor/child", true)?;
    let roots = copy_public_source(&fixture.root, &fixture.snapshot)?;
    assert_eq!(
        roots,
        [fixture.root.clone(), fixture.root.join("vendor/child")]
    );
    assert_eq!(
        fs::read_to_string(fixture.snapshot.join("vendor/child/source.rs"))?,
        "// public source\n"
    );
    assert!(!fixture.snapshot.join("vendor/child/.env").exists());
    assert!(!fixture.snapshot.join("vendor/child/.git").exists());
    Ok(())
}

#[test]
fn uninitialized_submodules_fail_closed_with_recovery_command() -> TestResult {
    let fixture = Fixture::new()?;
    fixture.submodule("child", false)?;
    let result = copy_public_source(&fixture.root, &fixture.snapshot);
    assert!(result.is_err());
    if let Err(error) = result {
        assert!(
            error
                .to_string()
                .contains("git submodule update --init --recursive")
        );
    }
    Ok(())
}

#[test]
fn missing_submodule_fails_closed() -> TestResult {
    let fixture = Fixture::new()?;
    fixture.submodule("child", false)?;
    fs::remove_dir(fixture.root.join("child"))?;
    assert!(copy_public_source(&fixture.root, &fixture.snapshot).is_err());
    Ok(())
}

#[test]
fn tracked_submodule_credentials_fail_before_copying_contents() -> TestResult {
    let fixture = Fixture::new()?;
    fixture.submodule("child", true)?;
    git(&fixture.root.join("child"), &["add", "--force", ".env"])?;
    let result = copy_public_source(&fixture.root, &fixture.snapshot);
    assert!(result.is_err());
    assert!(!fixture.snapshot.join("child/.env").exists());
    Ok(())
}

#[test]
fn unignored_keystores_and_password_files_fail_before_copying_contents() -> TestResult {
    for name in ["release.jks", "vendor/.pgpass", ".netrc"] {
        let fixture = Fixture::new()?;
        let path = fixture.root.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, "PRIVATE_FIXTURE_MUST_NOT_ENTER_SNAPSHOT")?;
        assert!(copy_public_source(&fixture.root, &fixture.snapshot).is_err());
        assert!(!fixture.snapshot.join(name).exists());
    }
    Ok(())
}

#[test]
fn symlink_submodules_fail_closed() -> TestResult {
    let fixture = Fixture::new()?;
    fixture.submodule("child", false)?;
    fs::remove_dir(fixture.root.join("child"))?;
    std::os::unix::fs::symlink(&fixture.snapshot, fixture.root.join("child"))?;
    assert!(copy_public_source(&fixture.root, &fixture.snapshot).is_err());
    Ok(())
}

#[test]
fn symlink_parent_directories_fail_closed() -> TestResult {
    let fixture = Fixture::new()?;
    fs::create_dir(fixture.root.join("source"))?;
    fs::write(fixture.root.join("source/file.rs"), "public fixture")?;
    git(&fixture.root, &["add", "source/file.rs"])?;
    fs::rename(
        fixture.root.join("source"),
        fixture.directory.join("outside"),
    )?;
    std::os::unix::fs::symlink(
        fixture.directory.join("outside"),
        fixture.root.join("source"),
    )?;
    assert!(copy_public_source(&fixture.root, &fixture.snapshot).is_err());
    Ok(())
}

#[test]
fn nested_submodule_source_and_history_are_included() -> TestResult {
    let fixture = Fixture::new()?;
    fixture.submodule("child", true)?;
    let child = fixture.root.join("child");
    let nested = child.join("nested");
    fs::create_dir(&nested)?;
    git(&nested, &["init", "--quiet"])?;
    fs::write(nested.join("public.txt"), "nested public source")?;
    git(&nested, &["add", "public.txt"])?;
    let revision = git(&fixture.root, &["rev-parse", "HEAD"])?;
    git(
        &child,
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            &format!("160000,{},nested", revision.trim()),
        ],
    )?;
    let roots = copy_public_source(&fixture.root, &fixture.snapshot)?;
    assert_eq!(roots.len(), 3);
    assert_eq!(roots[2], nested);
    assert!(fixture.snapshot.join("child/nested/public.txt").is_file());
    Ok(())
}

#[test]
fn regular_indexed_file_replaced_with_directory_fails_closed() -> TestResult {
    let fixture = Fixture::new()?;
    fs::remove_file(fixture.root.join("README.md"))?;
    fs::create_dir(fixture.root.join("README.md"))?;
    assert!(copy_public_source(&fixture.root, &fixture.snapshot).is_err());
    Ok(())
}
