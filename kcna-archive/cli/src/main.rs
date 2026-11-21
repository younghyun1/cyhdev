//! KCNA archive collector entry point.

mod archive;
mod cli;
mod database;
mod error;
mod fetch;
mod html;
mod importer;
mod manifest;
mod search_index;
mod shutdown;
mod storage;
mod verify;

use std::path::Path;

use clap::Parser;
use mimalloc::MiMalloc;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use crate::{
    cli::{Cli, Command, DatabaseCommand},
    error::{Error, Result},
};

// The workspace allocator; the collector follows the website's choice so one
// allocator is built and tuned for the whole tree.
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() {
    init_tracing();
    let args = Cli::parse();
    if let Err(error) = run(args).await {
        error!(error = %error, "Command failed");
        std::process::exit(1);
    }
}

async fn run(args: Cli) -> Result<()> {
    let snapshot = match args.snapshot_dir {
        Some(directory) => directory,
        None => args.data_dir.join("snapshot"),
    };
    match args.command {
        Command::Database(arguments) => {
            database_command(&args.data_dir, &snapshot, arguments.command).await
        }
        Command::Verify => {
            // Loading and searching are CPU-bound and synchronous.
            tokio::task::spawn_blocking(move || verify::run(&snapshot)).await?
        }
        collection => collect(&args.data_dir, collection).await,
    }
}

async fn collect(data_dir: &Path, command: Command) -> Result<()> {
    let summary = archive::run(data_dir, command).await?;
    info!(
        downloaded = summary.downloaded,
        reused = summary.reused,
        missing = summary.missing,
        failed = summary.failed,
        "Archive collection finished"
    );
    if summary.failed > 0 {
        return Err(Error::Incomplete {
            failures: summary.failed,
        });
    }
    Ok(())
}

async fn database_command(
    data_dir: &Path,
    snapshot: &Path,
    command: DatabaseCommand,
) -> Result<()> {
    match command {
        DatabaseCommand::Migrate => database::migrate::run(&database::pool::database_url()?).await,
        DatabaseCommand::Import(arguments) => {
            importer::runner::run(data_dir, snapshot, arguments).await
        }
        DatabaseCommand::Snapshot(arguments) => {
            let pool = database::pool::connect(&database::pool::database_url()?).await?;
            search_index::indexer::sync_and_report(&pool, snapshot, arguments.rebuild).await
        }
    }
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("kcna_archive=info,warn"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .json()
        .with_ansi(false)
        .flatten_event(true)
        .with_current_span(true)
        .init();
}
