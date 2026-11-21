//! Remembers pages the historical mirror reported as nonexistent.
//!
//! The mirror's index pages link to about twenty thousand pages it never
//! served. Without a record of them, every discovery run asks for each one
//! again. Entries expire so a page that appears later is still collected.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{File, OpenOptions},
    io::AsyncWriteExt,
    sync::Mutex,
};
use tracing::info;
use url::Url;

use crate::{
    error::{Result, io_error},
    storage,
};

/// A dead link is asked for again after this long. The mirror is a frozen
/// archive, so revivals are rare and a month of staleness costs nothing.
const RECHECK_AFTER_SECONDS: i64 = 30 * 24 * 60 * 60;

#[derive(Deserialize, Serialize)]
struct Record {
    url: String,
    checked_at: Timestamp,
}

/// Dead links known when the run started, plus an append-only log of new ones.
///
/// Lookups read only the starting set: a run schedules each URL once, so a
/// link recorded during the run is never looked up again in the same run.
pub struct DeadLinks {
    path: PathBuf,
    known: HashMap<String, Timestamp>,
    log: Mutex<File>,
}

impl DeadLinks {
    /// Loads unexpired entries and rewrites the log without the rest, so the
    /// file stays bounded by the number of dead links instead of by run count.
    /// `recheck` discards every entry, which makes the run ask for all of them.
    pub async fn open(data_dir: &Path, recheck: bool) -> Result<Self> {
        let path = data_dir.join("manifests/dead-links.jsonl");
        let stored = storage::read_if_exists(&path).await?.unwrap_or_default();
        let known = if recheck {
            HashMap::new()
        } else {
            unexpired(&stored, Timestamp::now())
        };
        storage::write_atomic(&path, &encode(&known)?).await?;
        let log = OpenOptions::new()
            .append(true)
            .open(&path)
            .await
            .map_err(io_error(&path))?;
        if !known.is_empty() {
            info!(
                dead_links = known.len(),
                "Skipping pages the mirror already reported as nonexistent"
            );
        }
        Ok(Self {
            path,
            known,
            log: Mutex::new(log),
        })
    }

    /// Reports whether the host recently said this page does not exist.
    pub fn contains(&self, url: &Url) -> bool {
        self.known.contains_key(url.as_str())
    }

    /// Appends one dead link. Each record is flushed on its own because an
    /// interrupted run must not lose the evidence it already paid requests for.
    pub async fn record(&self, url: &Url) -> Result<()> {
        let mut line = serde_json::to_vec(&Record {
            url: url.as_str().to_owned(),
            checked_at: Timestamp::now(),
        })?;
        line.push(b'\n');
        let mut log = self.log.lock().await;
        log.write_all(&line).await.map_err(io_error(&self.path))?;
        log.flush().await.map_err(io_error(&self.path))
    }
}

/// Keeps the newest check of each URL that is younger than the recheck age.
/// Malformed lines are dropped: the worst outcome is one repeated request.
fn unexpired(stored: &[u8], now: Timestamp) -> HashMap<String, Timestamp> {
    let mut known = HashMap::new();
    for line in stored.split(|byte| *byte == b'\n') {
        let Ok(record) = serde_json::from_slice::<Record>(line) else {
            continue;
        };
        let age = now
            .as_second()
            .saturating_sub(record.checked_at.as_second());
        if age >= RECHECK_AFTER_SECONDS {
            continue;
        }
        let newest = known.entry(record.url).or_insert(record.checked_at);
        if record.checked_at > *newest {
            *newest = record.checked_at;
        }
    }
    known
}

fn encode(known: &HashMap<String, Timestamp>) -> Result<Vec<u8>> {
    let mut encoded = Vec::with_capacity(known.len() * 96);
    for (url, checked_at) in known {
        serde_json::to_writer(
            &mut encoded,
            &Record {
                url: url.clone(),
                checked_at: *checked_at,
            },
        )?;
        encoded.push(b'\n');
    }
    Ok(encoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(second: i64) -> Result<Timestamp> {
        Timestamp::from_second(second)
            .map_err(|error| crate::error::Error::InvalidRecord(error.to_string()))
    }

    fn line(url: &str, checked_at: Timestamp) -> Result<Vec<u8>> {
        let mut line = serde_json::to_vec(&Record {
            url: url.to_owned(),
            checked_at,
        })?;
        line.push(b'\n');
        Ok(line)
    }

    #[test]
    fn drops_expired_entries_and_keeps_the_newest_check() -> Result<()> {
        let now = at(100 * RECHECK_AFTER_SECONDS)?;
        let fresh = at(now.as_second() - 60)?;
        let older = at(now.as_second() - 3600)?;
        let expired = at(now.as_second() - RECHECK_AFTER_SECONDS)?;
        let mut stored = Vec::new();
        stored.extend(line("http://www.kcna.co.jp/a.htm", older)?);
        stored.extend(line("http://www.kcna.co.jp/a.htm", fresh)?);
        stored.extend(line("http://www.kcna.co.jp/b.htm", expired)?);
        stored.extend(b"not json\n");
        let known = unexpired(&stored, now);
        assert_eq!(known.len(), 1);
        assert_eq!(known.get("http://www.kcna.co.jp/a.htm"), Some(&fresh));
        Ok(())
    }

    #[test]
    fn encoding_round_trips_through_the_loader() -> Result<()> {
        let now = at(100 * RECHECK_AFTER_SECONDS)?;
        let mut known = HashMap::new();
        known.insert(
            "http://www.kcna.co.jp/a.htm".to_owned(),
            at(now.as_second() - 5)?,
        );
        known.insert(
            "http://www.kcna.co.jp/b.htm".to_owned(),
            at(now.as_second() - 9)?,
        );
        assert_eq!(unexpired(&encode(&known)?, now), known);
        assert!(unexpired(b"", now).is_empty());
        Ok(())
    }
}
