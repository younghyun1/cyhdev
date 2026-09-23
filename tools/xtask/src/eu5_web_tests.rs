//! Submodule pinning checks against disposable local repositories.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

use super::eu5_web::{check_submodule_status, check_submodule_worktree, require_pinned_checkout};

type TestResult = Result<(), Box<dyn std::error::Error>>;
static NEXT_DIRECTORY: AtomicUsize = AtomicUsize::new(0);

const REQUIRED_FILES: [&str; 5] = [
    "Cargo.toml",
    "Cargo.lock",
    "web/index.html",
    "assets/eu5-locations.bitcode.zst",
    "assets/eu5-indexes.bitcode.zst",
];

/// A parent repository whose EU5 submodule is cloned from a local upstream.
struct Fixture {
    directory: PathBuf,
    root: PathBuf,
}

impl Fixture {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let id = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let directory =
            std::env::temp_dir().join(format!("cyhdev-eu5-pin-test-{}-{id}", std::process::id()));
        fs::create_dir(&directory)?;
        let fixture = Self {
            root: directory.join("root"),
            directory,
        };
        let upstream = fixture.directory.join("upstream");
        for relative in REQUIRED_FILES {
            let path = upstream.join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(path, "fixture\n")?;
        }
        git(&upstream, &["init", "--quiet"])?;
        git(&upstream, &["add", "."])?;
        commit(&upstream)?;

        fs::create_dir(&fixture.root)?;
        git(&fixture.root, &["init", "--quiet"])?;
        let upstream_path = upstream.to_str().ok_or("fixture path is not UTF-8")?;
        git(
            &fixture.root,
            &[
                "-c",
                "protocol.file.allow=always",
                "submodule",
                "add",
                "--quiet",
                upstream_path,
                "vendor/eu5-location-filter",
            ],
        )?;
        commit(&fixture.root)?;
        Ok(fixture)
    }

    fn submodule(&self) -> PathBuf {
        self.root.join("vendor/eu5-location-filter")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _cleanup_result = fs::remove_dir_all(&self.directory);
    }
}

fn git(directory: &Path, arguments: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(directory)
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

fn commit(directory: &Path) -> TestResult {
    git(
        directory,
        &[
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "--quiet",
            "--allow-empty",
            "-m",
            "fixture",
        ],
    )?;
    Ok(())
}

fn rejection(root: &Path) -> Result<String, Box<dyn std::error::Error>> {
    match require_pinned_checkout(root) {
        Ok(()) => Err("modified submodule checkout was accepted".into()),
        Err(error) => Ok(error.to_string()),
    }
}

#[test]
fn pinned_clean_submodule_is_accepted() -> TestResult {
    let fixture = Fixture::new()?;
    require_pinned_checkout(&fixture.root)?;
    Ok(())
}

#[test]
fn edited_or_untracked_submodule_files_are_rejected() -> TestResult {
    let fixture = Fixture::new()?;
    fs::write(fixture.submodule().join("Cargo.toml"), "edited\n")?;
    assert!(rejection(&fixture.root)?.contains("1 uncommitted or untracked path(s)"));

    git(
        &fixture.submodule(),
        &["checkout", "--quiet", "--", "Cargo.toml"],
    )?;
    fs::write(fixture.submodule().join("local.rs"), "// untracked\n")?;
    assert!(rejection(&fixture.root)?.contains("git -C vendor/eu5-location-filter status"));
    Ok(())
}

#[test]
fn submodule_moved_away_from_gitlink_is_rejected() -> TestResult {
    let fixture = Fixture::new()?;
    commit(&fixture.submodule())?;
    let message = rejection(&fixture.root)?;
    assert!(message.contains("vendor/eu5-location-filter is checked out at a commit other than"));
    assert!(message.contains("git submodule update --init --recursive"));
    Ok(())
}

#[test]
fn submodule_status_markers_map_to_recovery_steps() -> TestResult {
    let clean =
        " 2499f0f211ded9618c41e86c8a5dd43467328d7e vendor/eu5-location-filter (heads/main)\n";
    assert!(check_submodule_status(clean).is_ok());
    assert!(check_submodule_status("").is_ok());

    for (line, expected) in [
        (
            "-2499f0f211ded9618c41e86c8a5dd43467328d7e vendor/eu5-location-filter",
            "vendor/eu5-location-filter is not initialized",
        ),
        (
            "+1111111111111111111111111111111111111111 vendor/eu5-location-filter (heads/main)",
            "other than the recorded gitlink",
        ),
        (
            "U0000000000000000000000000000000000000000 vendor/eu5-location-filter",
            "has merge conflicts",
        ),
        ("?garbage", "unrecognized `git submodule status` line"),
    ] {
        match check_submodule_status(&format!("{clean}{line}\n")) {
            Ok(()) => return Err(format!("status line was accepted: {line}").into()),
            Err(error) => assert!(error.to_string().contains(expected), "{error}"),
        }
    }
    Ok(())
}

#[test]
fn submodule_worktree_accepts_only_empty_porcelain() {
    assert!(check_submodule_worktree("").is_ok());
    assert!(check_submodule_worktree("\n").is_ok());
    assert!(check_submodule_worktree(" M Cargo.toml\n?? local.rs\n").is_err());
}
