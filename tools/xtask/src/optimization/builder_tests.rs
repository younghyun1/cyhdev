//! Cache identity, compiler flags and shell delegation without optimized builds.

use std::{path::Path, process::Command};

use super::container;

fn arguments(command: &Command) -> Vec<&std::ffi::OsStr> {
    command.get_args().collect()
}

#[test]
fn same_compiler_reuses_cache_and_new_nightly_changes_the_builder_input() -> crate::TaskResult<()> {
    let root = Path::new("/repository with spaces");
    let build = |date| container::builder_command(root, "123", "synthetic-builder", date);
    let current = build(Some("nightly-2026-10-06"));
    assert_eq!(
        arguments(&current),
        arguments(&build(Some("nightly-2026-10-06")))
    );
    assert_ne!(
        arguments(&current),
        arguments(&build(Some("nightly-2026-10-07")))
    );
    let args = arguments(&current);
    assert!(args.windows(2).any(|pair| pair
        == [
            "--build-arg",
            "OPTIMIZATION_RUST_TOOLCHAIN=nightly-2026-10-06"
        ]));
    assert!(args.contains(&std::ffi::OsStr::new("--pull")));
    assert!(
        !args
            .iter()
            .any(|arg| arg.to_string_lossy().starts_with("--no-cache"))
    );
    assert_eq!(current.get_current_dir(), Some(root));
    let pinned = build(None);
    assert!(!arguments(&pinned).iter().any(|arg| {
        arg.to_string_lossy()
            .starts_with("OPTIMIZATION_RUST_TOOLCHAIN=")
    }));
    let dockerfile = include_str!("../../../../rust-be-template/Dockerfile");
    let stage = match dockerfile.split_once(" AS optimization-builder\n") {
        Some((_, stage)) => stage,
        None => return Err(crate::TaskError("missing optimization builder".into())),
    };
    let declaration = stage.find("ARG OPTIMIZATION_RUST_TOOLCHAIN=${RUST_VERSION}");
    let install = stage.find("rustup toolchain install \"$RUSTUP_TOOLCHAIN\"");
    let native_packages = stage.find("apt-get install");
    assert!(
        matches!((native_packages, declaration), (Some(packages), Some(declaration)) if packages < declaration)
    );
    assert!(
        matches!((declaration, install), (Some(declaration), Some(install)) if declaration < install)
    );
    assert!(stage.contains("RUSTUP_TOOLCHAIN=\"${OPTIMIZATION_RUST_TOOLCHAIN}\""));
    Ok(())
}

#[test]
fn pgo_stages_preserve_cpu_frame_pointer_and_relocation_flags() -> crate::TaskResult<()> {
    let run = Path::new("/native run");
    let baseline = container::encoded_flags(run, "baseline", "znver3")?;
    assert_eq!(
        baseline.split('\u{1f}').collect::<Vec<_>>(),
        [
            "-Ctarget-cpu=znver3",
            "-Cforce-frame-pointers=yes",
            "-Clink-arg=-Wl,--emit-relocs"
        ]
    );
    let instrumented = container::encoded_flags(run, "instrumented", "znver3")?;
    assert_eq!(
        instrumented,
        format!("{baseline}\u{1f}-Cprofile-generate=/native run/raw/pgo")
    );
    let pgo = container::encoded_flags(run, "pgo", "znver3")?;
    assert_eq!(
        pgo,
        format!(
            "{baseline}\u{1f}-Cprofile-use=/optimize/pgo.profdata\u{1f}-Cllvm-args=-pgo-warn-missing-function"
        )
    );
    assert!(container::encoded_flags(run, "unknown", "znver3").is_err());
    Ok(())
}

#[cfg(unix)]
#[test]
fn wrapper_preserves_root_arguments_freshness_request_and_exit_status()
-> Result<(), Box<dyn std::error::Error>> {
    use std::{fs, os::unix::fs::PermissionsExt};
    struct Fixture(std::path::PathBuf);
    impl Drop for Fixture {
        fn drop(&mut self) {
            if let Err(error) = fs::remove_dir_all(&self.0) {
                eprintln!("wrapper fixture cleanup failed: {error}");
            }
        }
    }
    let directory =
        std::env::temp_dir().join(format!("cyhdev-build-wrapper-{}", std::process::id()));
    fs::create_dir(&directory)?;
    let fixture = Fixture(directory);
    let root = fixture.0.join("repository with spaces");
    let bin = fixture.0.join("bin");
    fs::create_dir(&root)?;
    fs::create_dir(&bin)?;
    let script = root.join("build_pgo_and_bolt.sh");
    fs::write(&script, include_str!("../../../../build_pgo_and_bolt.sh"))?;
    fs::set_permissions(&script, fs::Permissions::from_mode(0o755))?;
    let cargo = bin.join("cargo");
    fs::write(
        &cargo,
        "#!/bin/sh\nprintf '%s\\n' \"$PWD\" \"$CYHDEV_OPT_LATEST_NIGHTLY\" \"$@\"\nexit 23\n",
    )?;
    fs::set_permissions(&cargo, fs::Permissions::from_mode(0o755))?;
    for (passed, expected) in [
        (vec![], vec![]),
        (
            vec!["inputs with spaces/config.json"],
            vec!["inputs with spaces/config.json"],
        ),
        (
            vec!["config.json", "unexpected"],
            vec!["config.json", "unexpected"],
        ),
    ] {
        let output = Command::new(&script)
            .args(&passed)
            .current_dir(&fixture.0)
            .env("PATH", format!("{}:/usr/bin:/bin", bin.display()))
            .env("CYHDEV_OPT_LATEST_NIGHTLY", "0")
            .output()?;
        assert_eq!(output.status.code(), Some(23));
        let stdout = String::from_utf8(output.stdout)?;
        let mut lines = stdout.lines();
        assert_eq!(lines.next(), root.to_str());
        assert_eq!(lines.next(), Some("1"));
        assert_eq!(
            lines.take(2).collect::<Vec<_>>(),
            ["xtask", "build-pgo-and-bolt"]
        );
        assert_eq!(stdout.lines().skip(4).collect::<Vec<_>>(), expected);
    }
    Ok(())
}
