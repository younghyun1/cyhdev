//! Development staging for the vendored EU5 Slint browser application.

use std::{fs, path::Path, process::Command};

use crate::{TaskError, TaskResult, run_command};

pub(super) fn run(root: &Path) -> TaskResult<()> {
    require_checkout(root)?;
    let application = root.join("solid-csr-spa-template/public/eu5-locations-db/app");
    if application.exists() {
        fs::remove_dir_all(&application).map_err(|error| {
            TaskError(format!(
                "could not clear staged EU5 browser assets at {}: {error}",
                application.display()
            ))
        })?;
    }
    let package = application.join("pkg");
    fs::create_dir_all(&package).map_err(|error| {
        TaskError(format!(
            "could not create EU5 browser asset directory {}: {error}",
            package.display()
        ))
    })?;
    let host_document = root.join("vendor/eu5-location-filter/web/index.html");
    fs::copy(&host_document, application.join("index.html")).map_err(|error| {
        TaskError(format!(
            "could not stage EU5 host document {}: {error}",
            host_document.display()
        ))
    })?;

    let mut command = command(root, &package);
    run_command(&mut command)
}

pub(crate) fn require_checkout(root: &Path) -> TaskResult<()> {
    let source = root.join("vendor/eu5-location-filter");
    for relative in [
        "Cargo.toml",
        "Cargo.lock",
        "web/index.html",
        "assets/eu5-locations.bitcode.zst",
        "assets/eu5-indexes.bitcode.zst",
    ] {
        let required = source.join(relative);
        if !required.is_file() {
            return Err(TaskError(format!(
                "EU5 source checkout is incomplete at {}; run `git submodule update --init --recursive` from {}",
                required.display(),
                root.display()
            )));
        }
    }
    Ok(())
}

pub(super) fn command(root: &Path, output: &Path) -> Command {
    let mut command = Command::new("wasm-pack");
    command
        .arg("build")
        .arg(root.join("vendor/eu5-location-filter"))
        .args([
            "--dev",
            "--target",
            "web",
            "--no-pack",
            "--no-typescript",
            "--out-dir",
        ])
        .arg(output)
        .args([
            "--",
            "--locked",
            "--no-default-features",
            "--features",
            "web",
        ])
        .current_dir(root);
    command
}
