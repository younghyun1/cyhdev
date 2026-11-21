//! Full-content importer for current multilingual kcna.kp pages.

use std::{path::Path, sync::Arc};

use futures::{StreamExt, stream};
use time::macros::{offset, time};
use tracing::{info, warn};

use crate::{
    cli::LiveEdition,
    database::{
        domain::{DocumentBody, DocumentBundle, DocumentTitle, ElementIndex, RawContent},
        pool::PgPool,
        store,
        types::PublicationPrecision,
        types::Source,
    },
    error::{Error, Result, io_error},
    html,
    importer::{
        common::{content_sha256, parse_parallelism, sha256, validated_url},
        extract, fs,
        live_dates::{Dates, html_files, load_dates},
        provenance::ProvenanceIndex,
        stats::ImportStats,
    },
};

/// Imports all downloaded language editions and their list-page provenance.
pub async fn run(
    data_dir: &Path,
    provenance: Arc<ProvenanceIndex>,
    pool: &PgPool,
    batch_size: usize,
    editions: &[LiveEdition],
) -> Result<ImportStats> {
    let mut stats = ImportStats::default();
    for edition in editions {
        let language = edition.path();
        let list_root = data_dir.join(format!("raw/kcna_kp/lists/{language}"));
        let dates = tokio::task::spawn_blocking(move || load_dates(&list_root)).await??;
        let dates = Arc::new(dates);
        let article_dir = data_dir.join(format!("raw/kcna_kp/articles/{language}"));
        let files = html_files(fs::files(&article_dir).await?);
        let mut results = stream::iter(files)
            .map(|path| {
                let provenance = Arc::clone(&provenance);
                let dates = Arc::clone(&dates);
                let edition = *edition;
                async move {
                    let worker_path = path.clone();
                    let built = tokio::task::spawn_blocking(move || {
                        build_bundle(&worker_path, &provenance, edition, &dates)
                    })
                    .await;
                    (path, built)
                }
            })
            .buffer_unordered(parse_parallelism());
        let mut batch = Vec::with_capacity(batch_size);
        while let Some((path, built)) = results.next().await {
            match built {
                Ok(Ok(bundle)) => batch.push(bundle),
                Ok(Err(error)) => {
                    stats.failed = stats.failed.saturating_add(1);
                    warn!(path = %path.display(), error = %error, "Live record skipped");
                }
                Err(error) => {
                    stats.failed = stats.failed.saturating_add(1);
                    warn!(path = %path.display(), error = %error, "Live record worker failed");
                }
            }
            if batch.len() >= batch_size {
                stats.record(store::upsert_batch(pool, &batch).await?);
                batch.clear();
            }
        }
        stats.record(store::upsert_batch(pool, &batch).await?);
        info!(
            language,
            stored = stats.stored,
            failed = stats.failed,
            "Live import progress"
        );
    }
    Ok(stats)
}

fn build_bundle(
    path: &Path,
    provenance: &ProvenanceIndex,
    edition: LiveEdition,
    dates: &Dates,
) -> Result<DocumentBundle> {
    let article_id = path
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| Error::InvalidRecord("article path is not UTF-8".to_owned()))?;
    let info = dates.get(article_id).ok_or_else(|| {
        Error::InvalidRecord(format!("publication date is missing for {article_id}"))
    })?;
    let bytes = std::fs::read(path).map_err(io_error(path))?;
    let article = extract::live(&html::decode(&bytes))?;
    let raw_sha256 = sha256(&bytes);
    let url = format!(
        "http://www.kcna.kp/{}/article/detail/{article_id}",
        edition.path()
    );
    let retrieval = provenance.resolve(path, raw_sha256, &url)?;
    // The database re-anchors day-precision records to noon in Asia/Pyongyang.
    let published_at = info.date.with_time(time!(12:00)).assume_offset(offset!(+9));
    Ok(DocumentBundle {
        source: Source::KcnaKp,
        source_record_id: Some(article_id.to_owned()),
        canonical_url: validated_url(&url)?,
        element_index: ElementIndex::try_new(0)
            .map_err(|error| Error::InvalidRecord(error.to_string()))?,
        language: edition.language(),
        translation_group_key: Some(format!("kcna.kp:{article_id}")),
        content_sha256: content_sha256(&article.title, Some(&article.body)),
        title: DocumentTitle::try_new(article.title)
            .map_err(|error| Error::InvalidRecord(error.to_string()))?,
        body: Some(
            DocumentBody::try_new(article.body)
                .map_err(|error| Error::InvalidRecord(error.to_string()))?,
        ),
        published_at,
        publication_precision: PublicationPrecision::Day,
        publication_date: info.date,
        published_at_raw: Some(info.raw_date.clone()),
        category: info.categories.first().cloned(),
        copyright_status: None,
        source_metadata: serde_json::json!({
            "article_id": article_id,
            "categories": info.categories,
            "category_ids": info.category_ids,
            "publication_date_raw": info.raw_date,
        }),
        retrieved_at: retrieval.retrieved_at,
        requested_url: validated_url(&retrieval.url)?,
        final_url: None,
        http_status: Some(200),
        media_type: Some("text/html".to_owned()),
        charset: Some(html::charset(&bytes)),
        raw_content: RawContent::try_new(bytes)
            .map_err(|error| Error::InvalidRecord(error.to_string()))?,
        raw_sha256,
        response_headers: serde_json::json!({}),
        manifest_metadata: retrieval.raw,
    })
}
