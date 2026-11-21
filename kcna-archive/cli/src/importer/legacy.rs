//! Full-content importer for indexed kcna.co.jp articles.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use futures::{StreamExt, stream};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{info, warn};

use crate::{
    database::{pool::PgPool, store},
    error::{Result, io_error},
    importer::{
        common::parse_parallelism,
        legacy_record::{self, PendingRecord, SeedRecord},
        provenance::ProvenanceIndex,
        stats::ImportStats,
    },
};

/// Records imported between progress log lines.
const PROGRESS_INTERVAL: usize = 10_000;

/// Which slice of the seed index to import.
#[derive(Clone, Copy, Debug)]
pub struct Window {
    /// Leading records to pass over when resuming.
    pub skip: usize,
    /// Records to import after the skipped ones; `None` imports the rest.
    pub limit: Option<usize>,
}

/// Imports the full source page selected by every public metadata record.
pub async fn run(
    data_dir: &Path,
    seed_index: &Path,
    provenance: Arc<ProvenanceIndex>,
    pool: &PgPool,
    batch_size: usize,
    window: Window,
) -> Result<ImportStats> {
    let file = tokio::fs::File::open(seed_index)
        .await
        .map_err(io_error(seed_index))?;
    let root = Arc::new(data_dir.join("raw/kcna_co_jp"));
    let mut lines = BufReader::new(file).lines();
    let mut batch = Vec::with_capacity(batch_size);
    let mut stats = ImportStats::default();
    let mut skipped = 0_usize;
    let mut taken = 0_usize;
    let mut next_progress = PROGRESS_INTERVAL;
    while let Some(line) = lines.next_line().await.map_err(io_error(seed_index))? {
        if skipped < window.skip {
            skipped += 1;
            continue;
        }
        if window.limit.is_some_and(|limit| taken >= limit) {
            break;
        }
        taken += 1;
        let raw = serde_json::from_str::<Value>(&line)?;
        let record = serde_json::from_value::<SeedRecord>(raw.clone())?;
        batch.push(PendingRecord { record, raw });
        if batch.len() >= batch_size {
            stats = stats + process_batch(&root, &provenance, pool, &mut batch).await?;
            if stats.stored + stats.failed >= next_progress {
                next_progress = stats.stored + stats.failed + PROGRESS_INTERVAL;
                info!(
                    stored = stats.stored,
                    failed = stats.failed,
                    "Historical import progress"
                );
            }
        }
    }
    stats = stats + process_batch(&root, &provenance, pool, &mut batch).await?;
    Ok(stats)
}

/// Parses each page of the batch on the blocking pool, then stores the batch
/// in one transaction.
async fn process_batch(
    root: &Arc<PathBuf>,
    provenance: &Arc<ProvenanceIndex>,
    pool: &PgPool,
    batch: &mut Vec<PendingRecord>,
) -> Result<ImportStats> {
    // Records sharing a page are grouped so the page is read and parsed once.
    let mut groups = HashMap::<String, Vec<PendingRecord>>::new();
    for pending in batch.drain(..) {
        groups
            .entry(pending.record.url.clone())
            .or_default()
            .push(pending);
    }
    let results = stream::iter(groups.into_values())
        .map(|records| {
            let root = Arc::clone(root);
            let provenance = Arc::clone(provenance);
            let size = records.len();
            async move {
                let built = tokio::task::spawn_blocking(move || {
                    legacy_record::build_group(&root, &provenance, records)
                })
                .await;
                match built {
                    Ok(group) => group,
                    Err(error) => {
                        warn!(error = %error, records = size, "Historical page worker failed");
                        (Vec::new(), size)
                    }
                }
            }
        })
        .buffer_unordered(parse_parallelism())
        .collect::<Vec<_>>()
        .await;
    let mut bundles = Vec::with_capacity(results.iter().map(|result| result.0.len()).sum());
    let mut failed = 0_usize;
    for (mut group_bundles, group_failed) in results {
        bundles.append(&mut group_bundles);
        failed = failed.saturating_add(group_failed);
    }
    let mut stats = ImportStats {
        failed,
        ..ImportStats::default()
    };
    stats.record(store::upsert_batch(pool, &bundles).await?);
    Ok(stats)
}
