//! PostgreSQL import orchestration.

use std::{path::Path, sync::Arc};

use tracing::info;

use crate::{
    cli::DatabaseImportArgs,
    database::{migrate, pool},
    error::{Error, Result},
    importer::{legacy, legacy_unlisted, live, provenance, stats::ImportStats, stiles},
};

/// Migrates the target and imports each requested source without external writes.
pub async fn run(data_dir: &Path, snapshot: &Path, args: DatabaseImportArgs) -> Result<()> {
    let database_url = pool::database_url()?;
    migrate::run(&database_url).await?;
    let pool = pool::connect(&database_url).await?;
    let provenance = Arc::new(provenance::load(data_dir).await?);
    let seed_index = args
        .seed_index
        .unwrap_or_else(|| data_dir.join("indexes/okhc/kcna_jp.jsonl"));
    let legacy_import = async {
        if args.skip_legacy {
            Ok(ImportStats::default())
        } else {
            let window = legacy::Window {
                skip: args.skip_legacy_records,
                limit: args.max_legacy_records,
            };
            let listed = legacy::run(
                data_dir,
                &seed_index,
                Arc::clone(&provenance),
                &pool,
                args.batch_size,
                window,
            )
            .await?;
            // Pages newer than the inventory are imported once the listed ones
            // are stored, so a later inventory can supersede them.
            let unlisted = legacy_unlisted::run(
                data_dir,
                &seed_index,
                Arc::clone(&provenance),
                &pool,
                args.batch_size,
            )
            .await?;
            Ok(listed + unlisted)
        }
    };
    let live_import = async {
        if args.skip_live {
            Ok(ImportStats::default())
        } else {
            live::run(
                data_dir,
                Arc::clone(&provenance),
                &pool,
                args.batch_size,
                &args.live_languages,
            )
            .await
        }
    };
    let stiles_import = async {
        if args.skip_stiles {
            Ok(ImportStats::default())
        } else {
            stiles::run(data_dir, &pool, args.batch_size).await
        }
    };
    let (legacy_stats, live_stats, stiles_stats) =
        tokio::try_join!(legacy_import, live_import, stiles_import)?;
    let stats = legacy_stats + live_stats + stiles_stats;
    info!(
        stored = stats.stored,
        changed = stats.changed,
        failed = stats.failed,
        "PostgreSQL import finished"
    );
    // Copies are relinked and the snapshot refreshed even after partial
    // failures, so everything that was stored is searchable.
    crate::importer::copies::refresh(&pool).await?;
    crate::search_index::indexer::sync_and_report(&pool, snapshot, false).await?;
    if stats.failed > 0 {
        return Err(Error::Incomplete {
            failures: stats.failed,
        });
    }
    Ok(())
}
