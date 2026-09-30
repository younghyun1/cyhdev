//! Live driver execution with exact artifact and campaign identities.

use std::process::Command;

use super::{coverage, files, measurement, process, state::State};
use crate::{TaskError, TaskResult};

impl State {
    fn command(
        &self,
        arguments: &[String],
        stage: &str,
        binary: &str,
        seed_stage: &str,
    ) -> TaskResult<Command> {
        let mut command = Command::new(&arguments[0]);
        command
            .args(&arguments[1..])
            .current_dir(&self.root)
            .env("CYHDEV_OPT_MODE", "training")
            .env("CYHDEV_OPT_STAGE", stage)
            .env("CYHDEV_OPT_RUN", &self.directory)
            .env("CYHDEV_OPT_CAMPAIGN", &self.config.campaign)
            .env("CYHDEV_OPT_RUNTIME", &self.config.runtime_directory)
            .env("CYHDEV_BINARY", self.directory.join(binary))
            .env(
                "CYHDEV_BINARY_SHA256",
                files::digest(&self.directory.join(binary))?,
            )
            .env(
                "CYHDEV_SEED_BINARY",
                self.directory.join(format!("{seed_stage}-minecraft-seed")),
            )
            .env("CYHDEV_CAMPAIGN_SHA256", &self.campaign_digest)
            .env(
                "CYHDEV_COVERAGE_REPORT",
                self.directory.join(format!("{stage}-coverage.json")),
            )
            .env(
                "CYHDEV_BENCHMARK_REPORT",
                self.directory.join(format!("{stage}-benchmark.json")),
            );
        if stage == "instrumented" {
            command.env(
                "LLVM_PROFILE_FILE",
                self.directory.join("raw/pgo/%m-%p.profraw"),
            );
        } else {
            command.env_remove("LLVM_PROFILE_FILE");
        }
        Ok(command)
    }

    pub(super) fn train(&mut self, stage: &str, binary: &str, seed_stage: &str) -> TaskResult<()> {
        self.check_inputs()?;
        println!("Optimization campaign: {stage} successful coverage");
        let digest = files::digest(&self.directory.join(binary))?;
        process::bounded(
            &mut self.command(&self.config.training_command, stage, binary, seed_stage)?,
            self.config.timeout_seconds,
        )?;
        if files::digest(&self.directory.join(binary))? != digest {
            return Err(TaskError("driver changed its binary".into()));
        }
        coverage::verify(
            &self.root,
            &self.directory,
            stage,
            &self.directory.join(binary),
            &self.config.campaign,
        )?;
        self.check_inputs()
    }

    pub(super) fn measurement(&self, stage: &str) -> TaskResult<measurement::Measurement> {
        files::read_json(
            &self.directory.join(format!("{stage}-benchmark.json")),
            1024 * 1024,
        )
    }

    pub(super) fn benchmark(&self, stage: &str) -> TaskResult<()> {
        self.check_inputs()?;
        println!("Optimization campaign: {stage} repeated benchmark");
        let binary = format!("{stage}-rust-be-template");
        let digest = files::digest(&self.directory.join(&binary))?;
        process::bounded(
            self.command(
                &self.config.benchmark_command,
                stage,
                &binary,
                if stage == "bolt" { "pgo" } else { stage },
            )?
            .env("CYHDEV_OPT_MODE", "benchmark"),
            self.config.timeout_seconds,
        )?;
        if files::digest(&self.directory.join(&binary))? != digest {
            return Err(TaskError("benchmark changed its binary".into()));
        }
        measurement::validate(&self.measurement(stage)?, stage, &digest)?;
        self.check_inputs()
    }
}
