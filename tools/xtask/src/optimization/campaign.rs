//! An isolated baseline, PGO, BOLT and acceptance sequence.

use std::{fs, path::Path};

use serde_json::json;

use crate::{TaskError, TaskResult};

use super::state::State;
use super::{artifacts, config::Config, container, files, inventory, measurement, process};

/// Inspect a campaign without compiling, or execute it from a clean commit.
pub(crate) fn run(root: &Path, arguments: &[String]) -> TaskResult<()> {
    if arguments.len() == 2 && arguments[0] == "inventory" {
        return inventory::generate(root, &root.join(&arguments[1]));
    }
    if arguments.len() != 2 || !matches!(arguments[0].as_str(), "plan" | "run") {
        return Err(TaskError(
            "usage: cargo xtask optimize inventory OUTPUT_DIR | <plan|run> CONFIG.json".into(),
        ));
    }
    let config_path = root.join(&arguments[1]);
    let config = Config::read(root, &config_path)?;
    if arguments[0] == "plan" {
        println!(
            "Campaign: {}; CPU: {}; runtime: {}\nStages: baseline build/coverage/benchmark; PGO instrument/train/merge/build/coverage/benchmark; BOLT instrument/train/merge/optimize/strip/coverage/benchmark; compare; export\nRequires a clean commit, Linux x86-64 runner, Docker, complete live fixtures and benchmark reports. Output: target/optimization/{}/dist/",
            config.name,
            config.target_cpu,
            config.runtime_directory.display(),
            config.name
        );
        return Ok(());
    }
    if !cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        return Err(TaskError("run the campaign on native Linux x86-64; emulation would distort training and benchmarks".into()));
    }
    let revision = process::revision(root)?;
    let directory = root.join("target/optimization").join(&config.name);
    let parent = match directory.parent() {
        Some(parent) => parent,
        None => return Err(TaskError("invalid run directory".into())),
    };
    match fs::create_dir_all(parent).and_then(|()| fs::create_dir(&directory)) {
        Ok(()) => {}
        Err(error) => {
            return Err(TaskError(format!(
                "cannot create fresh campaign; use a new name: {error}"
            )));
        }
    }
    let directory = match directory.canonicalize() {
        Ok(path) => path,
        Err(error) => return Err(TaskError(format!("cannot resolve run directory: {error}"))),
    };
    for kind in ["pgo", "bolt"] {
        if let Err(error) = fs::create_dir_all(directory.join(format!("raw/{kind}"))) {
            return Err(TaskError(format!(
                "cannot create profile directory: {error}"
            )));
        }
    }
    let epoch = crate::release::source_date_epoch(root)?;
    let mut state = State {
        root: root.to_path_buf(),
        directory,
        config,
        config_path,
        config_digest: String::new(),
        campaign_digest: String::new(),
        revision,
        image: String::new(),
        receipt: json!({"schema_version":1,"status":"active","stages":[]}),
    };
    state.config_digest = files::digest(&state.config_path)?;
    state.campaign_digest = files::digest(&state.config.campaign)?;
    state.receipt["revision"] = json!(state.revision);
    state.receipt["config_sha256"] = json!(state.config_digest);
    state.receipt["campaign_sha256"] = json!(state.campaign_digest);
    state.receipt["target_cpu"] = json!(state.config.target_cpu);
    state.receipt["source_date_epoch"] = json!(epoch);
    state.save()?;
    match state.execute(&epoch) {
        Ok(()) => {
            state.receipt["status"] = json!("accepted");
            state.save()?;
            println!(
                "Accepted PGO+BOLT artifact: {}",
                state.directory.join("dist/rust-be-template").display()
            );
            Ok(())
        }
        Err(error) => {
            state.receipt["status"] = json!("failed");
            state.save()?;
            Err(error)
        }
    }
}

impl State {
    fn execute(&mut self, epoch: &str) -> TaskResult<()> {
        inventory::export(&self.root, &self.directory.join("surface.json"))?;
        inventory::preflight(
            &self.root,
            &self.config.campaign,
            &self.directory.join("surface.json"),
        )?;
        self.image = container::prepare(&self.root, &self.directory, epoch, &self.config.name)?;
        self.receipt["builder_image"] = json!(self.image);
        self.receipt["toolchain_sha256"] =
            json!(files::digest(&self.directory.join("toolchain.txt"))?);
        self.receipt["packages_sha256"] =
            json!(files::digest(&self.directory.join("packages.txt"))?);
        self.check_inputs()?;
        for stage in ["baseline", "instrumented"] {
            container::compile(
                &self.image,
                &self.directory,
                stage,
                &self.config.target_cpu,
                epoch,
            )?;
            self.record(stage)?;
            self.train(stage, &format!("{stage}-rust-be-template"), stage)?;
            if stage == "baseline" {
                self.benchmark(stage)?;
            }
        }
        let profiles = artifacts::profiles(&self.directory, "pgo")?;
        container::merge_pgo(&self.image, &self.directory, &profiles)?;
        container::compile(
            &self.image,
            &self.directory,
            "pgo",
            &self.config.target_cpu,
            epoch,
        )?;
        self.record("pgo")?;
        self.train("pgo", "pgo-rust-be-template", "pgo")?;
        self.benchmark("pgo")?;
        let bolt_input = files::digest(&self.directory.join("pgo-rust-be-template"))?;
        self.receipt["bolt_input_sha256"] = json!(bolt_input);
        artifacts::instrument_bolt(&self.image, &self.directory)?;
        self.train(
            "bolt-instrumented",
            "bolt-instrumented-rust-be-template",
            "pgo",
        )?;
        if files::digest(&self.directory.join("pgo-rust-be-template"))? != bolt_input {
            return Err(TaskError(
                "BOLT input changed after profile collection".into(),
            ));
        }
        let profiles = artifacts::profiles(&self.directory, "bolt")?;
        artifacts::optimize_bolt(&self.image, &self.directory, &profiles)?;
        self.record("bolt")?;
        self.train("bolt", "bolt-rust-be-template", "pgo")?;
        self.benchmark("bolt")?;
        let baseline = self.measurement("baseline")?;
        let pgo = self.measurement("pgo")?;
        let bolt = self.measurement("bolt")?;
        for (reference, candidate) in [(&baseline, &pgo), (&baseline, &bolt), (&pgo, &bolt)] {
            measurement::compare(
                reference,
                candidate,
                self.config.maximum_latency_ratio,
                self.config.minimum_throughput_ratio,
            )?;
        }
        self.check_inputs()?;
        artifacts::publish(&self.root, &self.directory)?;
        self.receipt["artifact_sha256"] = json!(files::digest(
            &self.directory.join("dist/rust-be-template")
        )?);
        self.receipt["seed_sha256"] =
            json!(files::digest(&self.directory.join("dist/minecraft-seed"))?);
        Ok(())
    }

    pub(super) fn check_inputs(&self) -> TaskResult<()> {
        if process::revision(&self.root)? != self.revision
            || files::digest(&self.config_path)? != self.config_digest
            || files::digest(&self.config.campaign)? != self.campaign_digest
        {
            return Err(TaskError(
                "source or campaign changed during optimization".into(),
            ));
        }
        Ok(())
    }

    fn record(&mut self, stage: &str) -> TaskResult<()> {
        let stages = match self.receipt["stages"].as_array_mut() {
            Some(stages) => stages,
            None => return Err(TaskError("invalid internal receipt".into())),
        };
        stages.push(json!({"stage":stage,"binary_sha256":files::digest(&self.directory.join(format!("{stage}-rust-be-template")))?}));
        self.save()
    }

    fn save(&self) -> TaskResult<()> {
        files::write_json(&self.directory.join("receipt.json"), &self.receipt)
    }
}
