//! Binary, relocation, profile and provenance validation before export.

use std::{fs, path::Path};

use serde_json::json;

use crate::{TaskError, TaskResult, run_command};

use super::{container, files, process};

pub(super) const BOLT_EXECUTABLE: &str = "/usr/lib/llvm-19/bin/llvm-bolt";

pub(super) fn verify_elf(image: &str, run: &Path, name: &str, bolt: bool) -> TaskResult<()> {
    let binary = format!("/optimize/{name}");
    let header = process::output(container::base(image, run)?.args(["readelf", "-h", &binary]))?;
    let sections = process::output(container::base(image, run)?.args(["readelf", "-SW", &binary]))?;
    if !header.contains("Advanced Micro Devices X86-64")
        || !header.contains("DYN (Position-Independent Executable file)")
        || (!bolt && (!sections.contains(".symtab") || !sections.contains(".rela.text")))
        || (bolt && !sections.contains(".note.bolt_info"))
    {
        return Err(TaskError(format!(
            "{name} lacks required ELF architecture, PIE, symbols, relocations or BOLT provenance"
        )));
    }
    let libraries = process::output(container::base(image, run)?.args(["ldd", &binary]))?;
    if libraries.contains("not found") {
        return Err(TaskError(format!(
            "{name} has unresolved dynamic libraries"
        )));
    }
    files::write_json(
        &run.join(format!("{name}-elf.json")),
        &json!({"sha256":files::digest(&run.join(name))?,"header":header,"sections":sections,"libraries":libraries}),
    )
}

pub(super) fn profiles(run: &Path, kind: &str) -> TaskResult<Vec<String>> {
    let directory = run.join(format!("raw/{kind}"));
    let entries = match fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(error) => return Err(TaskError(format!("cannot read profiles: {error}"))),
    };
    let mut files = Vec::new();
    let mut total = 0u64;
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => return Err(TaskError(format!("cannot enumerate profiles: {error}"))),
        };
        let metadata = match entry.path().symlink_metadata() {
            Ok(meta) => meta,
            Err(error) => return Err(TaskError(format!("cannot inspect profile: {error}"))),
        };
        if !metadata.is_file() || metadata.len() == 0 {
            return Err(TaskError(
                "profile is empty, a symlink, or not a regular file".into(),
            ));
        }
        let filename = match entry.file_name().to_str() {
            Some(name)
                if name.bytes().all(|byte| {
                    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.')
                }) =>
            {
                name.to_owned()
            }
            _ => return Err(TaskError("invalid profile filename".into())),
        };
        if kind == "pgo" && !filename.ends_with(".profraw") {
            return Err(TaskError(
                "PGO directory contains a non-profraw file".into(),
            ));
        }
        total = total.saturating_add(metadata.len());
        files.push(format!("/optimize/raw/{kind}/{filename}"));
        if files.len() > 4096 || total > 4 * 1024 * 1024 * 1024 {
            return Err(TaskError("profiles exceed 4096 files or 4 GiB".into()));
        }
    }
    if files.is_empty() {
        return Err(TaskError(
            "training produced no profiles; check graceful exit and profile paths".into(),
        ));
    }
    files.sort();
    let mut receipts = Vec::new();
    for file in &files {
        let relative = file.trim_start_matches("/optimize/");
        receipts.push(json!({"path":relative,"sha256":files::digest(&run.join(relative))?}));
    }
    files::write_json(&run.join(format!("{kind}-profiles.json")), &receipts)?;
    Ok(files)
}

pub(super) fn instrument_bolt(image: &str, run: &Path) -> TaskResult<()> {
    run_command(container::base(image, run)?.args([
        BOLT_EXECUTABLE,
        "/optimize/pgo-rust-be-template",
        "-instrument",
        "-runtime-instrumentation-lib=libbolt_rt_instr.a",
        "-o",
        "/optimize/bolt-instrumented-rust-be-template",
        &format!(
            "-instrumentation-file={}/raw/bolt/profile.fdata",
            run.display()
        ),
        "-instrumentation-file-append-pid",
    ]))
}

pub(super) fn optimize_bolt(image: &str, run: &Path, profiles: &[String]) -> TaskResult<()> {
    let combined = match fs::File::create(run.join("bolt.fdata")) {
        Ok(file) => file,
        Err(error) => {
            return Err(TaskError(format!(
                "cannot create merged BOLT profile: {error}"
            )));
        }
    };
    run_command(
        container::base(image, run)?
            .arg("merge-fdata-19")
            .args(profiles)
            .stdout(combined),
    )?;
    run_command(container::base(image, run)?.args([
        BOLT_EXECUTABLE,
        "/optimize/pgo-rust-be-template",
        "-o",
        "/optimize/bolt-unstripped",
        "-data=/optimize/bolt.fdata",
        "-reorder-blocks=ext-tsp",
        "-reorder-functions=cdsort",
        "-split-functions",
        "-split-all-cold",
        "-split-eh",
        "-dyno-stats",
    ]))?;
    files::copy(
        &run.join("bolt-unstripped"),
        &run.join("bolt-rust-be-template"),
    )?;
    run_command(container::base(image, run)?.args([
        "strip",
        "--strip-all",
        "/optimize/bolt-rust-be-template",
    ]))?;
    verify_elf(image, run, "bolt-rust-be-template", true)
}

pub(super) fn publish(root: &Path, run: &Path) -> TaskResult<()> {
    let dist = run.join("dist");
    match fs::create_dir(&dist) {
        Ok(()) => {}
        Err(error) => {
            return Err(TaskError(format!(
                "cannot create artifact directory: {error}"
            )));
        }
    }
    for (source, name) in [
        ("bolt-rust-be-template", "rust-be-template"),
        ("pgo-minecraft-seed", "minecraft-seed"),
    ] {
        files::copy(&run.join(source), &dist.join(name))?;
    }
    let licenses = dist.join("minecraft-seed-licenses");
    match fs::create_dir(&licenses) {
        Ok(()) => {}
        Err(error) => {
            return Err(TaskError(format!(
                "cannot create license directory: {error}"
            )));
        }
    }
    for name in ["LICENSE", "THIRD_PARTY_NOTICES.md"] {
        files::copy(
            &root.join("tools/minecraft-seed").join(name),
            &licenses.join(name),
        )?;
    }
    Ok(())
}
