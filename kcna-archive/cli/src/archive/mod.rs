//! Source-specific archive collectors.

mod dead_links;
mod legacy;
#[cfg(test)]
mod legacy_tests;
mod live;
mod live_lists;

use std::{path::Path, sync::Arc, time::Duration};

use tracing::error;

use crate::{
    cli::{Command, CommonArgs},
    error::{Error, Result},
    fetch::Fetcher,
    manifest::Manifest,
    shutdown,
};

pub use legacy::run as run_legacy;
pub use live::run as run_live;

/// Aggregate collection totals.
#[derive(Clone, Copy, Debug, Default)]
pub struct Summary {
    pub downloaded: usize,
    pub reused: usize,
    /// Pages the host reports as nonexistent (HTTP 404 or 410).
    pub missing: usize,
    pub failed: usize,
}

impl Summary {
    /// Counts one page that could not be collected.
    pub fn record_error(&mut self, error: &Error) {
        if error.is_missing() {
            self.missing += 1;
        } else {
            self.failed += 1;
        }
    }
}

impl std::ops::Add for Summary {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            downloaded: self.downloaded + rhs.downloaded,
            reused: self.reused + rhs.reused,
            missing: self.missing + rhs.missing,
            failed: self.failed + rhs.failed,
        }
    }
}

/// Runs the selected collectors, flushing retrieval evidence on every exit path.
pub async fn run(data_dir: &Path, command: Command) -> Result<Summary> {
    let manifest = Arc::new(Manifest::open(data_dir).await?);
    // The collection future is moved into `select!` so an interrupt drops it,
    // and with it any manifest lock held mid-write, before the final flush.
    let outcome = tokio::select! {
        result = collect(data_dir, Arc::clone(&manifest), command) => result,
        () = shutdown::signal() => Err(Error::Interrupted),
    };
    let flushed = manifest.flush().await;
    let summary = outcome?;
    flushed?;
    Ok(summary)
}

async fn collect(data_dir: &Path, manifest: Arc<Manifest>, command: Command) -> Result<Summary> {
    match command {
        Command::All(arguments) => run_all(data_dir, manifest, arguments).await,
        Command::Live(arguments) => {
            let fetcher = live_fetcher(arguments.concurrency)?;
            run_live(data_dir, manifest, fetcher, arguments.languages).await
        }
        Command::Legacy(arguments) => {
            let fetcher = legacy_fetcher(arguments.concurrency)?;
            run_legacy(
                data_dir,
                manifest,
                fetcher,
                arguments.seed_index,
                arguments.recheck_missing,
            )
            .await
        }
        Command::Database(_) | Command::Verify => Err(Error::InvalidRecord(
            "database commands do not collect pages".to_owned(),
        )),
    }
}

/// Runs both sources to completion. The hosts fail independently, so one
/// unreachable site must not cancel hours of progress on the other.
async fn run_all(data_dir: &Path, manifest: Arc<Manifest>, args: CommonArgs) -> Result<Summary> {
    let live = run_live(
        data_dir,
        Arc::clone(&manifest),
        live_fetcher(args.live_concurrency)?,
        args.languages,
    );
    let legacy = run_legacy(
        data_dir,
        manifest,
        legacy_fetcher(args.legacy_concurrency)?,
        args.seed_index,
        args.recheck_missing,
    );
    let (live, legacy) = tokio::join!(live, legacy);
    Ok(settle("kcna.kp", live) + settle("kcna.co.jp", legacy))
}

/// Converts a collector-level failure into one counted failure.
fn settle(source: &str, result: Result<Summary>) -> Summary {
    match result {
        Ok(summary) => summary,
        Err(error) => {
            error!(source, error = %error, "Collector stopped early");
            Summary {
                failed: 1,
                ..Summary::default()
            }
        }
    }
}

fn live_fetcher(concurrency: usize) -> Result<Arc<Fetcher>> {
    Ok(Arc::new(Fetcher::new(
        concurrency,
        Duration::from_millis(100),
    )?))
}

fn legacy_fetcher(concurrency: usize) -> Result<Arc<Fetcher>> {
    Ok(Arc::new(Fetcher::new(
        concurrency,
        Duration::from_millis(20),
    )?))
}
