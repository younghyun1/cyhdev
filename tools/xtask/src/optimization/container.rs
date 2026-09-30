//! A single immutable builder and fixed container paths for every compilation.

use std::{path::Path, process::Command};

use crate::{TaskError, TaskResult, run_command};

use super::process;

pub(super) const TARGET: &str = "x86_64-unknown-linux-gnu";

pub(super) fn prepare(root: &Path, run: &Path, epoch: &str, name: &str) -> TaskResult<String> {
    let tag = format!("cyhdev-optimization:{name}");
    run_command(
        Command::new("docker")
            .args([
                "build",
                "--pull",
                "--platform",
                "linux/amd64",
                "--file",
                "rust-be-template/Dockerfile",
                "--target",
                "optimization-builder",
                "--tag",
                &tag,
                "--build-arg",
                &format!("APP_BUILD_EPOCH={epoch}"),
                ".",
            ])
            .current_dir(root),
    )?;
    let image = process::output(
        Command::new("docker").args(["image", "inspect", "--format", "{{.Id}}", &tag]),
    )?;
    if !image.starts_with("sha256:") || image.len() != 71 {
        return Err(TaskError(
            "Docker did not return an immutable image identity".into(),
        ));
    }
    run_command(base(&image, run)?.args(["sh", "-eu", "-c", TOOLCHAIN_RECEIPT]))?;
    Ok(image)
}

const TOOLCHAIN_RECEIPT: &str = r#"
rustc -Vv > /optimize/toolchain.txt
host=$(rustc -Vv | sed -n 's/^host: //p')
profdata="$(rustc --print sysroot)/lib/rustlib/$host/bin/llvm-profdata"
test -x "$profdata"
"$profdata" --version >> /optimize/toolchain.txt
llvm-bolt-19 --version >> /optimize/toolchain.txt
sha256sum "$(command -v llvm-bolt-19)" "$profdata" >> /optimize/toolchain.txt
dpkg-query -W > /optimize/packages.txt
"#;

/// Network is disabled after locked dependencies and assets enter the image.
pub(super) fn base(image: &str, run: &Path) -> TaskResult<Command> {
    let run_path = match run.to_str() {
        Some(path) if !path.contains(',') => path,
        _ => {
            return Err(TaskError(
                "optimization path must be UTF-8 without commas".into(),
            ));
        }
    };
    let mut command = Command::new("docker");
    command.args(["run", "--rm", "--network", "none"]);
    #[cfg(unix)]
    {
        let uid = process::output(Command::new("id").arg("-u"))?;
        let gid = process::output(Command::new("id").arg("-g"))?;
        command.args(["--user", &format!("{uid}:{gid}")]);
    }
    command.args([
        "--mount",
        &format!("type=bind,source={run_path},target=/optimize"),
        "--workdir",
        "/workspace",
        image,
    ]);
    Ok(command)
}

pub(super) fn compile(
    image: &str,
    run: &Path,
    stage: &str,
    cpu: &str,
    epoch: &str,
) -> TaskResult<()> {
    let mut flags =
        format!("-Ctarget-cpu={cpu} -Cforce-frame-pointers=yes -Clink-arg=-Wl,--emit-relocs");
    match stage {
        "baseline" => {}
        "instrumented" => flags.push_str(" -Cprofile-generate=/optimize/raw/pgo"),
        "pgo" => flags.push_str(
            " -Cprofile-use=/optimize/pgo.profdata -Cllvm-args=-pgo-warn-missing-function",
        ),
        _ => return Err(TaskError("unknown compilation stage".into())),
    }
    // All variants share CPU, linker, standard-library, LTO and panic settings.
    run_command(base(image, run)?.args([
        "env",
        &format!("RUSTFLAGS={flags}"),
        &format!("CFLAGS=-march={cpu} -O3"),
        "CARGO_PROFILE_RELEASE_STRIP=false",
        "CARGO_INCREMENTAL=0",
        &format!("APP_BUILD_EPOCH={epoch}"),
        "cargo",
        "build",
        "--offline",
        "--locked",
        "--release",
        "--package",
        "rust-be-template",
        "--package",
        "minecraft-seed",
        "--bin",
        "rust-be-template",
        "--bin",
        "minecraft-seed",
        "--target",
        TARGET,
        "--target-dir",
        &format!("/optimize/cargo-{stage}"),
        "-Z",
        "build-std=core,alloc,std,panic_unwind",
        "-Z",
        "build-std-features=backtrace,panic-unwind",
    ]))?;
    for binary in ["rust-be-template", "minecraft-seed"] {
        super::files::copy(
            &run.join(format!("cargo-{stage}/{TARGET}/release/{binary}")),
            &run.join(format!("{stage}-{binary}")),
        )?;
    }
    super::artifacts::verify_elf(image, run, &format!("{stage}-rust-be-template"), false)
}

pub(super) fn merge_pgo(image: &str, run: &Path, profiles: &[String]) -> TaskResult<()> {
    let script = r#"host=$(rustc -Vv | sed -n 's/^host: //p'); exec "$(rustc --print sysroot)/lib/rustlib/$host/bin/llvm-profdata" merge --failure-mode=any -o /optimize/pgo.profdata "$@""#;
    run_command(
        base(image, run)?
            .args(["sh", "-eu", "-c", script, "profdata"])
            .args(profiles),
    )
}
