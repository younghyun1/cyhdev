//! Bounded lookup of append-only retrieval manifest entries.

use std::{collections::HashMap, path::Path};

use serde::Deserialize;
use serde_json::Value;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::warn;

use crate::{
    error::{Error, Result, io_error},
    manifest,
};

const MAX_MANIFEST_ENTRIES: usize = 2_000_000;

#[derive(Clone, Debug)]
pub struct Provenance {
    pub retrieved_at: OffsetDateTime,
    pub url: String,
    pub sha256: [u8; 32],
    pub raw: Value,
}

/// Latest manifest entry per raw file, keyed independently of where the data
/// directory lives.
#[derive(Debug, Default)]
pub struct ProvenanceIndex {
    records: HashMap<String, Provenance>,
}

#[derive(Debug, Deserialize)]
struct ManifestRecord {
    retrieved_at: String,
    url: String,
    path: String,
    sha256: String,
}

/// Loads the corpus manifest. A corpus without one still imports, with file
/// modification times standing in for retrieval times.
pub async fn load(data_dir: &Path) -> Result<ProvenanceIndex> {
    let path = data_dir.join("manifests/retrievals.jsonl");
    let file = match tokio::fs::File::open(&path).await {
        Ok(file) => file,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
            warn!(path = %path.display(), "Retrieval manifest is missing");
            return Ok(ProvenanceIndex::default());
        }
        Err(source) => return Err(io_error(&path)(source)),
    };
    let mut lines = BufReader::new(file).lines();
    let mut index = ProvenanceIndex {
        records: HashMap::with_capacity(400_000),
    };
    let mut line_number = 0_usize;
    while let Some(line) = lines.next_line().await.map_err(io_error(&path))? {
        line_number += 1;
        match parse_line(&line) {
            Ok((key, provenance)) => index.insert(key, provenance),
            // A process killed mid-write can leave one torn line; it must not
            // make the rest of the manifest unusable.
            Err(error) => {
                warn!(line = line_number, error = %error, "Skipping unreadable manifest entry");
            }
        }
        if index.records.len() > MAX_MANIFEST_ENTRIES {
            return Err(Error::CrawlLimit {
                limit: MAX_MANIFEST_ENTRIES,
            });
        }
    }
    Ok(index)
}

fn parse_line(line: &str) -> Result<(String, Provenance)> {
    let raw = serde_json::from_str::<Value>(line)?;
    let record = serde_json::from_value::<ManifestRecord>(raw.clone())?;
    let retrieved_at = OffsetDateTime::parse(&record.retrieved_at, &Rfc3339)?;
    let sha256 = digest(&record.sha256)?;
    Ok((
        manifest::corpus_key(Path::new(&record.path)),
        Provenance {
            retrieved_at,
            url: record.url,
            sha256,
            raw,
        },
    ))
}

impl ProvenanceIndex {
    fn insert(&mut self, key: String, provenance: Provenance) {
        match self.records.get_mut(&key) {
            Some(current) if provenance.retrieved_at > current.retrieved_at => {
                *current = provenance;
            }
            Some(_) => {}
            None => {
                self.records.insert(key, provenance);
            }
        }
    }

    /// Resolves exact provenance, recording an honest file-mtime fallback after
    /// local change. Blocking: call from the parsing workers, not the runtime.
    pub fn resolve(
        &self,
        path: &Path,
        actual_sha256: [u8; 32],
        fallback_url: &str,
    ) -> Result<Provenance> {
        let Some(recorded) = self.records.get(&manifest::corpus_key(path)) else {
            warn!(path = %path.display(), "Retrieval manifest entry is missing; using file modification provenance");
            return file_provenance(path, actual_sha256, fallback_url, None);
        };
        if recorded.sha256 == actual_sha256 {
            return Ok(recorded.clone());
        }
        warn!(path = %path.display(), "Raw file changed after its manifest entry; using file modification provenance");
        file_provenance(path, actual_sha256, &recorded.url, Some(&recorded.raw))
    }
}

fn file_provenance(
    path: &Path,
    actual_sha256: [u8; 32],
    url: &str,
    prior_manifest: Option<&Value>,
) -> Result<Provenance> {
    let metadata = std::fs::metadata(path).map_err(io_error(path))?;
    let retrieved_at = metadata
        .modified()
        .map(OffsetDateTime::from)
        .map_err(io_error(path))?;
    Ok(Provenance {
        retrieved_at,
        url: url.to_owned(),
        sha256: actual_sha256,
        raw: serde_json::json!({
            "recovered_from_file": true,
            "file_modified_at": retrieved_at.to_string(),
            "actual_sha256": hex::encode(actual_sha256),
            "prior_manifest": prior_manifest,
        }),
    })
}

fn digest(value: &str) -> Result<[u8; 32]> {
    let decoded = hex::decode(value)?;
    decoded.try_into().map_err(|bytes: Vec<u8>| {
        Error::InvalidRecord(format!("SHA-256 has {} bytes instead of 32", bytes.len()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST: &str = "721f03393a995cb9b35a756f9b1dea29881ec4d195a215d16ebe523c1de6dff1";

    fn line(path: &str, retrieved_at: &str) -> String {
        format!(
            r#"{{"retrieved_at":"{retrieved_at}","source":"kcna.co.jp","url":"http://www.kcna.co.jp/a.htm","path":"{path}","bytes":1,"sha256":"{DIGEST}"}}"#
        )
    }

    #[test]
    fn resolves_entries_regardless_of_data_directory_spelling() -> Result<()> {
        let mut index = ProvenanceIndex::default();
        let (key, provenance) = parse_line(&line(
            "data/raw/kcna_co_jp/a.htm",
            "2026-08-31T01:36:48.816429899Z",
        ))?;
        index.insert(key, provenance);
        let sha256 = digest(DIGEST)?;
        for path in [
            "/srv/corpus/raw/kcna_co_jp/a.htm",
            "./data/raw/kcna_co_jp/a.htm",
        ] {
            let resolved = index.resolve(Path::new(path), sha256, "http://fallback/")?;
            assert_eq!(resolved.url, "http://www.kcna.co.jp/a.htm");
            assert_eq!(resolved.retrieved_at.year(), 2026);
        }
        Ok(())
    }

    #[test]
    fn keeps_the_latest_entry_and_rejects_torn_lines() -> Result<()> {
        let mut index = ProvenanceIndex::default();
        for stamp in [
            "2026-09-02T00:00:00Z",
            "2026-11-21T00:00:00Z",
            "2026-08-31T00:00:00Z",
        ] {
            let (key, provenance) = parse_line(&line("raw/kcna_co_jp/a.htm", stamp))?;
            index.insert(key, provenance);
        }
        let latest = index.records.get("raw/kcna_co_jp/a.htm");
        assert!(latest.is_some_and(|entry| u8::from(entry.retrieved_at.month()) == 11));
        assert!(parse_line(r#"{"retrieved_at":"2026-09-02T00:00"#).is_err());
        Ok(())
    }
}
