//! Fixture-driven throughput measurement and regression checking.

mod check;
mod cli;
mod config;
mod config_validation;
mod error;
mod executor;
mod hardware;
mod measurement;
mod report;
mod statistics;

use std::process::ExitCode;

use cli::{CliAction, Command};
use error::{HarnessError, HarnessResult};
use tracing::{error, info};
use tracing_subscriber::util::SubscriberInitExt;

fn main() -> ExitCode {
    let result = match initialize_logging() {
        Ok(()) => run(),
        Err(error) => Err(error),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            error!(error = %error, retryable = error.is_retryable(), "Throughput harness failed");
            ExitCode::FAILURE
        }
    }
}

fn initialize_logging() -> HarnessResult<()> {
    tracing_subscriber::fmt()
        .json()
        .with_ansi(false)
        .with_current_span(true)
        .with_span_list(false)
        .flatten_event(true)
        .with_max_level(tracing::Level::INFO)
        .with_writer(std::io::stderr)
        .finish()
        .try_init()
        .map_err(|source| HarnessError::Logging(source.to_string()))
}

fn run() -> HarnessResult<()> {
    match cli::parse()? {
        CliAction::Help => {
            cli::print_help();
            Ok(())
        }
        CliAction::Command(Command::Run(options)) => {
            let report = measurement::run(&options)?;
            report::write(&options.output, &report)?;
            report::print_json(&report)?;
            check::enforce(&report.verdict)?;
            info!(
                workload = %report.workload.name,
                requests = report.metrics.requests,
                throughput_rps = report.metrics.throughput_requests_per_second,
                "Throughput workload passed its regression thresholds"
            );
            Ok(())
        }
        CliAction::Command(Command::Check(options)) => {
            let checked = check::check_saved_report(&options)?;
            report::print_json(&checked)?;
            check::enforce(&checked.verdict)
        }
    }
}
