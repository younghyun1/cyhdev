//! Loads a snapshot the way the website does and reports what it holds.
//!
//! This is the check to run on a snapshot before or after copying it to a
//! server: it needs no database, reads the directory into memory exactly as
//! the backend will, and times a fixed set of requests against it.

use std::{path::Path, time::Instant};

use kcna_archive_core::service::{
    archive::{Archive, ServiceError},
    params::QueryParams,
};
use tracing::info;

use crate::error::{Error, Result};

/// Requests timed against the loaded snapshot: browsing, a Korean stem, a
/// single character (the costliest clause, a prefix scan), a phrase across a
/// particle boundary, and words in three other scripts.
const PROBES: [&str; 7] = [
    "",
    "김정은",
    "핵",
    "로동당 대회",
    "peace talks",
    "朝鲜劳动党",
    "ракета",
];

/// Loads the snapshot in `snapshot` into memory and logs its size, the load
/// time, the resident memory of this process, and per-request timings.
pub fn run(snapshot: &Path) -> Result<()> {
    let started = Instant::now();
    let archive = Archive::load_into_memory(snapshot)?;
    let load_seconds = started.elapsed().as_secs_f64();
    let status = archive.status().map_err(service_error)?;
    info!(
        snapshot = %snapshot.display(),
        load_seconds,
        memory_bytes = archive.memory_bytes(),
        resident_bytes = resident_bytes(),
        indexed = status.indexed,
        documents = status.documents,
        copies = status.copies,
        translation_groups = status.translation_groups,
        oldest_date = status.oldest_date,
        newest_date = status.newest_date,
        snapshot_built_at = %status.snapshot_built_at,
        "Snapshot loaded into memory"
    );
    for text in PROBES {
        let params = QueryParams {
            q: text.to_owned(),
            ..QueryParams::default()
        };
        let started = Instant::now();
        let page = archive.search(&params).map_err(service_error)?;
        let search_ms = started.elapsed().as_secs_f64() * 1_000.0;
        let started = Instant::now();
        let histogram = archive.histogram(&params).map_err(service_error)?;
        let histogram_ms = started.elapsed().as_secs_f64() * 1_000.0;
        // Opening the first result exercises the stored-document path.
        let started = Instant::now();
        if let Some(first) = page.items.first() {
            archive.document(&first.id, text).map_err(service_error)?;
        }
        let document_ms = started.elapsed().as_secs_f64() * 1_000.0;
        info!(
            query = text,
            total = page.total,
            buckets = histogram.buckets.len(),
            search_ms,
            histogram_ms,
            document_ms,
            "Probe request answered"
        );
    }
    if status.indexed == 0 {
        return Err(Error::InvalidRecord(
            "snapshot holds no documents".to_owned(),
        ));
    }
    Ok(())
}

fn service_error(error: ServiceError) -> Error {
    match error {
        ServiceError::Unavailable(error) => Error::Snapshot(error),
        ServiceError::BadRequest | ServiceError::NotFound => {
            Error::InvalidRecord(format!("probe request was rejected: {error}"))
        }
    }
}

/// Resident set size of this process, where the platform exposes it.
fn resident_bytes() -> Option<u64> {
    let status = std::fs::read_to_string("/proc/self/status").ok()?;
    let line = status.lines().find(|line| line.starts_with("VmRSS:"))?;
    let kibibytes = line.split_whitespace().nth(1)?.parse::<u64>().ok()?;
    Some(kibibytes * 1024)
}
