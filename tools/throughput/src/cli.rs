//! Minimal command-line parsing with repository-relative defaults.

use std::{env, path::PathBuf};

use crate::error::{HarnessError, HarnessResult};

const DEFAULT_WORKLOAD: &str = "tools/throughput/workloads/public-read-v1.json";
const DEFAULT_ENVIRONMENT: &str = "tools/throughput/config/development.json";
const DEFAULT_THRESHOLDS: &str = "tools/throughput/config/fixture-thresholds.json";
const DEFAULT_OUTPUT: &str = "target/throughput/public-read-v1.json";

#[derive(Debug)]
pub enum CliAction {
    Help,
    Command(Command),
}

#[derive(Debug)]
pub enum Command {
    Run(RunOptions),
    Check(CheckOptions),
}

#[derive(Debug)]
pub struct RunOptions {
    pub workload: PathBuf,
    pub environment: PathBuf,
    pub thresholds: PathBuf,
    pub output: PathBuf,
    pub target: Option<String>,
}

#[derive(Debug)]
pub struct CheckOptions {
    pub report: PathBuf,
    pub thresholds: PathBuf,
}

pub fn parse() -> HarnessResult<CliAction> {
    let mut arguments = env::args().skip(1).peekable();
    let action = match arguments.peek().map(String::as_str) {
        Some("help" | "--help" | "-h") => return Ok(CliAction::Help),
        Some("check") => {
            let _command = arguments.next();
            Command::Check(parse_check(arguments.collect())?)
        }
        Some("run") => {
            let _command = arguments.next();
            Command::Run(parse_run(arguments.collect())?)
        }
        Some(value) if value.starts_with('-') => Command::Run(parse_run(arguments.collect())?),
        Some(value) => {
            return Err(HarnessError::Arguments(format!(
                "unknown command `{value}`"
            )));
        }
        None => Command::Run(default_run_options()),
    };
    Ok(CliAction::Command(action))
}

fn parse_run(arguments: Vec<String>) -> HarnessResult<RunOptions> {
    let mut options = default_run_options();
    let mut index = 0;
    while index < arguments.len() {
        let flag = &arguments[index];
        index += 1;
        let value = match arguments.get(index) {
            Some(value) => value.clone(),
            None => {
                return Err(HarnessError::Arguments(format!(
                    "missing value for `{flag}`"
                )));
            }
        };
        index += 1;
        match flag.as_str() {
            "--workload" => options.workload = PathBuf::from(value),
            "--environment" => options.environment = PathBuf::from(value),
            "--thresholds" => options.thresholds = PathBuf::from(value),
            "--output" => options.output = PathBuf::from(value),
            "--target" => options.target = Some(value),
            unknown => {
                return Err(HarnessError::Arguments(format!(
                    "unknown run option `{unknown}`"
                )));
            }
        }
    }
    Ok(options)
}

fn parse_check(arguments: Vec<String>) -> HarnessResult<CheckOptions> {
    let mut report = PathBuf::from(DEFAULT_OUTPUT);
    let mut thresholds = PathBuf::from(DEFAULT_THRESHOLDS);
    let mut index = 0;
    while index < arguments.len() {
        let flag = &arguments[index];
        index += 1;
        let value = match arguments.get(index) {
            Some(value) => value.clone(),
            None => {
                return Err(HarnessError::Arguments(format!(
                    "missing value for `{flag}`"
                )));
            }
        };
        index += 1;
        match flag.as_str() {
            "--report" => report = PathBuf::from(value),
            "--thresholds" => thresholds = PathBuf::from(value),
            unknown => {
                return Err(HarnessError::Arguments(format!(
                    "unknown check option `{unknown}`"
                )));
            }
        }
    }
    Ok(CheckOptions { report, thresholds })
}

fn default_run_options() -> RunOptions {
    RunOptions {
        workload: PathBuf::from(DEFAULT_WORKLOAD),
        environment: PathBuf::from(DEFAULT_ENVIRONMENT),
        thresholds: PathBuf::from(DEFAULT_THRESHOLDS),
        output: PathBuf::from(DEFAULT_OUTPUT),
        target: None,
    }
}

pub fn print_help() {
    println!(
        "Usage:\n  cargo xtask throughput [run options]\n  cargo run --locked --package throughput-harness -- check [options]\n\nRun options:\n  --workload PATH     Recorded workload JSON\n  --environment PATH  Declared environment JSON\n  --thresholds PATH   Regression thresholds JSON\n  --output PATH       Report destination\n  --target URL        Optional http:// target; defaults to the in-process fixture\n\nCheck options:\n  --report PATH       Existing report JSON\n  --thresholds PATH   Regression thresholds JSON"
    );
}
