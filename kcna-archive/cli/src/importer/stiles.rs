//! Importer for the recent public English cross-check dataset.

use std::{collections::HashSet, path::Path};

use serde::Deserialize;
use serde_json::Value;
use time::{
    OffsetDateTime,
    macros::{offset, time},
};
use tracing::{info, warn};
use url::Url;

use crate::{
    database::{
        domain::{DocumentBody, DocumentBundle, DocumentTitle, ElementIndex, RawContent},
        pool::PgPool,
        store,
        types::{Language, PublicationPrecision, Source},
    },
    error::{Error, Result, io_error},
    importer::{
        common::{content_sha256, sha256, validated_url},
        stats::ImportStats,
    },
};

const SNAPSHOT_URL: &str = "https://stilesdata.com/north-korea-news/headlines.json";
const MAX_RECORDS: usize = 100_000;

#[derive(Debug, Deserialize)]
struct StilesRecord {
    topic: Option<String>,
    headline: String,
    link: String,
    date: i64,
    story_text: Option<String>,
}

/// Imports each unique source link while preserving its complete JSON object.
pub async fn run(data_dir: &Path, pool: &PgPool, batch_size: usize) -> Result<ImportStats> {
    let path = data_dir.join("indexes/stiles/headlines.json");
    let bytes = tokio::fs::read(&path).await.map_err(io_error(&path))?;
    let metadata = tokio::fs::metadata(&path).await.map_err(io_error(&path))?;
    let retrieved_at = metadata
        .modified()
        .map(OffsetDateTime::from)
        .map_err(io_error(&path))?;
    // Parsing and search analysis of the whole snapshot are CPU-bound.
    let (bundles, failed) =
        tokio::task::spawn_blocking(move || build_bundles(&bytes, retrieved_at)).await??;
    let mut stats = ImportStats {
        failed,
        ..ImportStats::default()
    };
    for batch in bundles.chunks(batch_size) {
        stats.record(store::upsert_batch(pool, batch).await?);
    }
    info!(
        stored = stats.stored,
        failed = stats.failed,
        "Recent cross-check import progress"
    );
    Ok(stats)
}

fn build_bundles(
    bytes: &[u8],
    retrieved_at: OffsetDateTime,
) -> Result<(Vec<DocumentBundle>, usize)> {
    let records = serde_json::from_slice::<Vec<Value>>(bytes)?;
    if records.len() > MAX_RECORDS {
        return Err(Error::CrawlLimit { limit: MAX_RECORDS });
    }
    let snapshot_sha256 = sha256(bytes);
    // Keyed on the normalized URL, which is the stored natural key; two
    // spellings of one link in a batch would otherwise abort its upsert.
    let mut seen = HashSet::with_capacity(records.len());
    let mut bundles = Vec::with_capacity(records.len());
    let mut failed = 0_usize;
    for raw in records {
        let built = serde_json::from_value::<StilesRecord>(raw.clone())
            .map_err(Error::from)
            .and_then(|record| bundle(record, raw, retrieved_at, snapshot_sha256));
        match built {
            Ok(bundle) => {
                if seen.insert(bundle.canonical_url.as_ref().to_owned()) {
                    bundles.push(bundle);
                }
            }
            Err(error) => {
                failed = failed.saturating_add(1);
                warn!(error = %error, "Recent cross-check record skipped");
            }
        }
    }
    Ok((bundles, failed))
}

fn bundle(
    record: StilesRecord,
    raw_metadata: Value,
    retrieved_at: OffsetDateTime,
    snapshot_sha256: [u8; 32],
) -> Result<DocumentBundle> {
    let raw_content = serde_json::to_vec(&raw_metadata)?;
    let raw_sha256 = sha256(&raw_content);
    let source_time = OffsetDateTime::from_unix_timestamp(record.date.div_euclid(1_000))
        .map_err(|error| Error::InvalidRecord(error.to_string()))?;
    let publication_date = source_time.date();
    let published_at = publication_date
        .with_time(time!(12:00))
        .assume_offset(offset!(+9));
    let url = Url::parse(&record.link)?;
    let title = DocumentTitle::try_new(record.headline)
        .map_err(|error| Error::InvalidRecord(error.to_string()))?;
    let body = record
        .story_text
        .filter(|value| !value.trim().is_empty())
        .map(DocumentBody::try_new)
        .transpose()
        .map_err(|error| Error::InvalidRecord(error.to_string()))?;
    let source_record_id = format!("stiles:{}", hex::encode(raw_sha256));
    Ok(DocumentBundle {
        source: Source::StilesArchive,
        source_record_id: Some(source_record_id),
        canonical_url: validated_url(url.as_str())?,
        element_index: ElementIndex::try_new(0)
            .map_err(|error| Error::InvalidRecord(error.to_string()))?,
        language: Language::English,
        translation_group_key: translation_group(&url),
        content_sha256: content_sha256(title.as_ref(), body.as_ref().map(DocumentBody::as_ref)),
        title,
        body,
        published_at,
        publication_precision: PublicationPrecision::Day,
        publication_date,
        published_at_raw: Some(record.date.to_string()),
        category: record.topic,
        copyright_status: None,
        source_metadata: raw_metadata,
        retrieved_at,
        requested_url: validated_url(SNAPSHOT_URL)?,
        final_url: Some(validated_url(SNAPSHOT_URL)?),
        http_status: Some(200),
        media_type: Some("application/json".to_owned()),
        charset: Some("utf-8".to_owned()),
        raw_content: RawContent::try_new(raw_content)
            .map_err(|error| Error::InvalidRecord(error.to_string()))?,
        raw_sha256,
        response_headers: serde_json::json!({}),
        manifest_metadata: serde_json::json!({
            "snapshot_sha256": hex::encode(snapshot_sha256),
            "snapshot_url": SNAPSHOT_URL,
        }),
    })
}

fn translation_group(url: &Url) -> Option<String> {
    url.path_segments()
        .and_then(Iterator::last)
        .filter(|value| !value.is_empty())
        .map(|value| value.trim_end_matches(".kcmsf"))
        .map(|value| format!("kcna.kp:{value}"))
}
