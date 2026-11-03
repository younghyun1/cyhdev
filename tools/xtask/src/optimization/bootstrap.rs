//! Default operator build: fresh synthetic fixtures, then the existing accepted campaign.

use super::{
    bootstrap_inputs::{self, Inputs},
    bootstrap_resources::{self, Database, OwnedChild, Signals},
    campaign, inventory, process,
};
use crate::{TaskError, TaskResult};
use std::{
    env, fs,
    net::TcpStream,
    path::Path,
    process::Command,
    thread,
    time::{Duration, Instant},
};

pub(crate) fn run(root: &Path, arguments: &[String]) -> TaskResult<()> {
    match arguments {
        [] => managed(root, None),
        [flag, previous] if flag == "--rebenchmark" => managed(root, Some(&root.join(previous))),
        [argument] if matches!(argument.as_str(), "--help" | "-h") => {
            println!(
                "Usage: ./build_pgo_and_bolt.sh [CONFIG.json | --rebenchmark RUN_DIR]\nWithout a config: prepare fresh isolated PostgreSQL and synthetic providers, generate a unique campaign, run PGO+BOLT, and stop owned fixtures.\nWith a config: run manually prepared fixtures.\nWith --rebenchmark: prepare the same fresh fixtures and compare an earlier run's baseline, PGO and BOLT binaries in rotated rounds without compiling them.\nRequires native Linux x86-64, local Docker, PostgreSQL 18 client tools, OpenSSL, Node/npm, curl and oha. Existing services are never stopped."
            );
            Ok(())
        }
        [config] => campaign::run(root, &["run".into(), config.clone()]),
        _ => Err(TaskError(
            "usage: ./build_pgo_and_bolt.sh [CONFIG.json | --rebenchmark RUN_DIR]".into(),
        )),
    }
}

/// Runs a fresh campaign, or with `previous` re-benchmarks that run's binaries.
fn managed(root: &Path, previous: Option<&Path>) -> TaskResult<()> {
    if !cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        return Err(TaskError(
            "managed PGO+BOLT builds require native Linux x86-64".into(),
        ));
    }
    process::revision(root)?;
    if let Some(previous) = previous
        && !previous.join("surface.json").is_file()
    {
        return Err(TaskError(format!(
            "{} is not a campaign run directory with surface.json",
            previous.display()
        )));
    }
    let oha = bootstrap_inputs::prerequisites(root)?;
    let inputs = bootstrap_inputs::inputs(root, oha)?;
    let _signals = Signals::install()?;
    println!("Preparing fresh optimization campaign: {}", inputs.name);
    fs::create_dir_all(root.join("target/optimization-inputs"))
        .map_err(|error| TaskError(format!("cannot create campaign input parent: {error}")))?;
    fs::create_dir(&inputs.directory).map_err(|error| {
        TaskError(format!(
            "cannot create fresh campaign input directory: {error}"
        ))
    })?;
    browser_tools(root)?;
    let fixture = build_tools(root, &inputs)?;
    match previous {
        // The earlier run's surface matches its binaries, and reusing it avoids
        // the release-profile inventory export.
        Some(previous) => {
            fs::copy(
                previous.join("surface.json"),
                inputs.directory.join("surface.json"),
            )
            .map_err(|error| TaskError(format!("cannot copy the earlier surface: {error}")))?;
        }
        None => inventory::generate(root, &inputs.directory)?,
    }
    let mut directory = fs::DirBuilder::new();
    use std::os::unix::fs::DirBuilderExt;
    directory
        .mode(0o700)
        .create(&inputs.runtime)
        .map_err(|error| TaskError(format!("cannot create fresh runtime: {error}")))?;
    let database = Database::new(root, &inputs.runtime);
    let outcome = execute(root, &inputs, &database, &fixture, previous);
    let cleanup = database.cleanup();
    match (outcome, cleanup) {
        (Ok(()), Ok(())) => {
            fs::remove_dir_all(&inputs.runtime).map_err(|error| {
                TaskError(format!("cannot remove completed fixture runtime: {error}"))
            })?;
            Ok(())
        }
        (Err(error), Ok(())) => {
            eprintln!(
                "Private fixture diagnostics retained: {}",
                inputs.runtime.display()
            );
            Err(error)
        }
        (outcome, Err(error)) => Err(TaskError(format!(
            "fixture cleanup failed: {error}; campaign result: {outcome:?}"
        ))),
    }
}

fn build_tools(root: &Path, inputs: &Inputs) -> TaskResult<std::path::PathBuf> {
    let receipt = inputs.directory.join("fixture-build.jsonl");
    let log = fs::File::create(&receipt)
        .map_err(|error| TaskError(format!("cannot create fixture build receipt: {error}")))?;
    stage(fixture_build_command(root).stdout(log))?;
    // Fixture generation records its executable path as the reset hook.
    // Use Cargo's artifact JSON rather than assuming a conventional target layout.
    let metadata = fs::metadata(&receipt)
        .map_err(|error| TaskError(format!("cannot inspect fixture build receipt: {error}")))?;
    if metadata.len() > 16 * 1024 * 1024 {
        return Err(TaskError("fixture build receipt exceeds 16 MiB".into()));
    }
    let output = fs::read_to_string(&receipt)
        .map_err(|error| TaskError(format!("cannot read fixture build receipt: {error}")))?;
    artifact(&output)
}

pub(super) fn fixture_build_command(root: &Path) -> Command {
    let mut command = bootstrap_inputs::command("cargo", root);
    command.args([
        "build",
        "--locked",
        "--release",
        "--package",
        "optimization-fixtures",
        "--message-format=json-render-diagnostics",
    ]);
    command
}

pub(super) fn artifact(output: &str) -> TaskResult<std::path::PathBuf> {
    for line in output.lines() {
        let value: serde_json::Value = serde_json::from_str(line)
            .map_err(|error| TaskError(format!("invalid Cargo artifact receipt: {error}")))?;
        if value["reason"] == "compiler-artifact"
            && value["target"]["name"] == "optimization-fixtures"
            && let Some(path) = value["executable"].as_str()
        {
            return Ok(std::path::PathBuf::from(path));
        }
    }
    Err(TaskError(
        "Cargo did not report an optimization-fixtures executable".into(),
    ))
}

fn browser_tools(root: &Path) -> TaskResult<()> {
    let frontend = root.join("solid-csr-spa-template");
    if !frontend
        .join("node_modules/@playwright/test/package.json")
        .is_file()
    {
        stage(bootstrap_inputs::command("npm", &frontend).arg("ci"))?;
    }
    let available = bootstrap_inputs::command("node", &frontend).args(["--input-type=module", "-e", "import {chromium} from '@playwright/test'; import {existsSync} from 'node:fs'; process.exit(existsSync(chromium.executablePath()) ? 0 : 2);"]).status().map_err(|error| TaskError(format!("cannot check Playwright Chromium: {error}")))?;
    if !available.success() {
        stage(bootstrap_inputs::command("npm", &frontend).args([
            "exec",
            "--",
            "playwright",
            "install",
            "chromium",
        ]))?;
    }
    Ok(())
}

fn execute(
    root: &Path,
    inputs: &Inputs,
    database: &Database,
    fixture: &Path,
    previous: Option<&Path>,
) -> TaskResult<()> {
    stage(bootstrap_inputs::command("docker", root).args(["pull", "postgres:18"]))?;
    let image = process::output(bootstrap_inputs::command("docker", root).args([
        "image",
        "inspect",
        "--format",
        "{{.Id}}",
        "postgres:18",
    ]))?;
    stage(&mut database.start_command(&format!("cyhdev-opt-{}", inputs.name), &image))?;
    bootstrap_resources::wait_database(root)?;
    stage(
        fixture_command(root, inputs, fixture)
            .arg("prepare")
            .arg(&inputs.runtime),
    )?;
    stage(
        fixture_command(root, inputs, fixture)
            .arg("campaign")
            .arg(&inputs.runtime)
            .arg(&inputs.directory)
            .arg(&inputs.name)
            .arg(&inputs.cpu),
    )?;
    let log = fs::File::create(inputs.runtime.join("fixture-server.log"))
        .map_err(|error| TaskError(format!("cannot create fixture log: {error}")))?;
    let stderr = log
        .try_clone()
        .map_err(|error| TaskError(format!("cannot duplicate fixture log: {error}")))?;
    let mut server = OwnedChild::spawn(
        fixture_command(root, inputs, fixture)
            .arg("serve")
            .arg(&inputs.runtime)
            .stdout(log)
            .stderr(stderr),
    )?;
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if bootstrap_resources::interrupted() || !server.alive()? {
            return Err(TaskError(
                "fixture server stopped before readiness; inspect fixture-server.log".into(),
            ));
        }
        if [34901, 3465, 18555]
            .into_iter()
            .all(|port| TcpStream::connect(("127.0.0.1", port)).is_ok())
            && inputs.runtime.join("control.sock").exists()
            && inputs.runtime.join("world.sock").exists()
        {
            break;
        }
        if Instant::now() >= deadline {
            return Err(TaskError(
                "fixture server readiness timed out; inspect fixture-server.log".into(),
            ));
        }
        thread::sleep(Duration::from_millis(100));
    }
    let executable = env::current_exe()
        .map_err(|error| TaskError(format!("cannot resolve xtask executable: {error}")))?;
    let mut command = bootstrap_inputs::command("sh", root);
    command
        .args([
            "-eu",
            "-c",
            ". \"$1\"; shift; exec \"$@\"",
            "optimization-environment",
        ])
        .arg(inputs.runtime.join("environment.sh"))
        .arg(executable)
        .arg("optimize")
        .arg(if previous.is_some() {
            "rebenchmark"
        } else {
            "run"
        })
        .arg(inputs.directory.join("config.json"));
    if let Some(previous) = previous {
        command.arg(previous);
    }
    command
        .env("CYHDEV_OPT_LATEST_NIGHTLY", "1")
        .env("CYHDEV_OHA", &inputs.oha);
    stage(&mut command)
}

fn fixture_command(root: &Path, inputs: &Inputs, executable: &Path) -> Command {
    let mut command = bootstrap_inputs::command(executable, root);
    command.env("CYHDEV_OPT_DISPOSABLE", "1").env(
        "DB_URL",
        format!(
            "postgres://optimization_fixture:optimization-fixture-postgres@127.0.0.1:35432/{}",
            inputs.database
        ),
    );
    command
}

fn stage(command: &mut Command) -> TaskResult<()> {
    OwnedChild::spawn(command)?.wait()
}
