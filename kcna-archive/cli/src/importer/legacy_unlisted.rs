//! Importer for historical article pages that the public inventory does not list.
//!
//! The inventory ends in June 2025 while the mirror kept publishing until
//! October 2025, and a discovery crawl stores those later pages. Their file
//! names follow the same scheme as the listed ones, so the publication date
//! and language are read from the path and stored as `inferred_day`.

use std::{path::Path, sync::Arc};

use futures::{StreamExt, stream};
use tracing::{info, warn};

use crate::{
    database::{domain::DocumentBundle, pool::PgPool, store, types::PublicationPrecision},
    error::Result,
    importer::{
        common::parse_parallelism,
        legacy_page,
        legacy_record::{self, Descriptor},
        legacy_unlisted_scan::{Unlisted, find_unlisted},
        provenance::ProvenanceIndex,
        stats::ImportStats,
    },
};

/// Imports every stored article page absent from the inventory.
pub async fn run(
    data_dir: &Path,
    seed_index: &Path,
    provenance: Arc<ProvenanceIndex>,
    pool: &PgPool,
    batch_size: usize,
) -> Result<ImportStats> {
    let root = Arc::new(data_dir.join("raw/kcna_co_jp"));
    let scan_root = Arc::clone(&root);
    let seed_index = seed_index.to_owned();
    let unlisted =
        tokio::task::spawn_blocking(move || find_unlisted(&scan_root, &seed_index)).await??;
    info!(
        pages = unlisted.len(),
        "Importing historical pages absent from the inventory"
    );
    let mut results = stream::iter(unlisted)
        .map(|page| {
            let root = Arc::clone(&root);
            let provenance = Arc::clone(&provenance);
            async move {
                let relative = page.relative.clone();
                let built =
                    tokio::task::spawn_blocking(move || build_page(&root, &provenance, &page))
                        .await;
                (relative, built)
            }
        })
        .buffer_unordered(parse_parallelism());
    let mut stats = ImportStats::default();
    let mut batch = Vec::with_capacity(batch_size);
    while let Some((relative, built)) = results.next().await {
        match built {
            Ok(Ok(bundles)) => batch.extend(bundles),
            Ok(Err(error)) => {
                stats.failed = stats.failed.saturating_add(1);
                warn!(page = relative, error = %error, "Unlisted historical page skipped");
            }
            Err(error) => {
                stats.failed = stats.failed.saturating_add(1);
                warn!(page = relative, error = %error, "Unlisted historical page worker failed");
            }
        }
        if batch.len() >= batch_size {
            stats.record(store::upsert_batch(pool, &batch).await?);
            batch.clear();
        }
    }
    stats.record(store::upsert_batch(pool, &batch).await?);
    Ok(stats)
}

fn build_page(
    root: &Path,
    provenance: &ProvenanceIndex,
    unlisted: &Unlisted,
) -> Result<Vec<DocumentBundle>> {
    let url = format!("http://www.kcna.co.jp/{}", unlisted.relative);
    let page = legacy_page::load_page(root, provenance, &url)?;
    (0..page.article_count())
        .map(|element_index| {
            legacy_record::build_bundle(
                &page,
                Descriptor {
                    source_record_id: None,
                    language: unlisted.language,
                    element_index,
                    publication_date: unlisted.date,
                    publication_precision: PublicationPrecision::InferredDay,
                    published_at_raw: None,
                    copyright_status: None,
                    metadata: serde_json::json!({
                        "_kcna_archive": {
                            "listed_in_inventory": false,
                            "date_inferred_from": "url",
                        },
                    }),
                },
            )
        })
        .collect()
}
