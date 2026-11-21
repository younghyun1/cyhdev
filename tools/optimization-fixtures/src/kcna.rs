//! Synthetic KCNA archive snapshot written beside the other disposable runtime data.
//!
//! The campaign backend runs with the runtime directory as its working
//! directory and `KCNA_ARCHIVE_DIR` pointing at `data/kcna_archive` below it
//! (see `environment.rs`). Every fixture reset rebuilds that directory from the
//! invented corpus in `kcna_corpus.rs`, using only the public API of
//! `kcna-archive-core`, so the backend loads it at startup exactly like a real
//! snapshot.

use std::{fs, io, path::Path};

use anyhow::Context;
use kcna_archive_core::{
    error::search_error,
    search::{
        fields,
        request::CopyRef,
        writer::{self, IndexedDocument},
    },
    search_text,
    snapshot::{
        layout::index_directory,
        manifest::{FORMAT, Manifest},
    },
    types::{Language, PublicationPrecision, Source},
};
use serde_json::{Value, json};
use time::Date;
use uuid::Uuid;

use crate::kcna_corpus::{
    COPY_ID, ENGLISH_EDITION, ENTRIES, Entry, GROUP_ID, ID_BASE, KOREAN_EDITION,
};

/// Snapshot directory below the runtime directory.
const SNAPSHOT_PATH: &str = "data/kcna_archive";
/// Fixed so a rebuilt manifest is byte-identical.
const BUILT_AT: &str = "2026-11-21T00:00:00Z";
const LAST_RETRIEVED_AT: &str = "2026-11-20T00:00:00Z";
const INDEX_HEAP_BYTES: usize = 20_000_000;

/// Identifier of the document with a translation and a copy; the document page
/// of the browser census opens it.
pub fn document_id() -> Uuid {
    Uuid::from_u128(ID_BASE + ENGLISH_EDITION)
}

fn entry_id(entry: &Entry) -> Uuid {
    Uuid::from_u128(ID_BASE + entry.number)
}

/// Whether the shared analyzer finds `word` in the entry's title or body.
fn mentions(entry: &Entry, word: &str) -> bool {
    let mut found = false;
    for text in [entry.title, entry.body] {
        search_text::document_lexemes(text, |lexeme| found |= lexeme == word);
    }
    found
}

/// Values the scenario asserts, derived from the corpus so they cannot drift
/// from it. The English counts use the analyzer's lowercase words; the Korean
/// count is a substring match, as the analyzer indexes Hangul as bigrams.
pub fn parameters() -> Value {
    let count = |word: &str| ENTRIES.iter().filter(|entry| mentions(entry, word)).count();
    let mut years: Vec<i32> = ENTRIES.iter().map(|entry| entry.date.0).collect();
    years.dedup();
    json!({
        "documentId": document_id().to_string(),
        "kcna_translation_id": Uuid::from_u128(ID_BASE + KOREAN_EDITION).to_string(),
        "kcna_indexed": ENTRIES.len(),
        "kcna_lantern_total": count("lantern"),
        "kcna_korean_lantern_total": ENTRIES
            .iter()
            .filter(|entry| entry.language == Language::Korean
                && (entry.title.contains("등불") || entry.body.contains("등불")))
            .count(),
        "kcna_years": years.len(),
    })
}

/// Adds the archive's scenario parameters to the fixture receipt's parameters.
pub fn with_parameters(mut base: Value) -> anyhow::Result<Value> {
    let (Some(target), Value::Object(extra)) = (base.as_object_mut(), parameters()) else {
        anyhow::bail!("fixture parameters must be a JSON object");
    };
    target.extend(extra);
    Ok(base)
}

fn indexed(entry: &Entry) -> anyhow::Result<IndexedDocument> {
    let (year, month, day) = entry.date;
    let id = entry_id(entry);
    let pair = [ENGLISH_EDITION, KOREAN_EDITION].contains(&entry.number);
    let copies = if entry.number == ENGLISH_EDITION {
        vec![CopyRef {
            id: Uuid::from_u128(COPY_ID),
            source: Source::KcnaCoJp,
            url: "https://example.invalid/synthetic/copy/1".to_owned(),
        }]
    } else {
        Vec::new()
    };
    Ok(IndexedDocument {
        id,
        title: entry.title.to_owned(),
        body: Some(entry.body.to_owned()),
        date: Date::from_calendar_date(year, month, day)?,
        precision: PublicationPrecision::Day,
        language: entry.language,
        source: entry.source,
        canonical_url: format!("https://example.invalid/synthetic/{}", entry.number),
        category: Some(entry.category.to_owned()),
        source_record_id: Some(format!("SYN-{:03}", entry.number)),
        metadata: json!({"synthetic": true}),
        translation_group: pair.then(|| Uuid::from_u128(GROUP_ID)),
        copies,
    })
}

/// Writes the snapshot into `directory`, which must not hold an index yet.
pub fn build(directory: &Path) -> anyhow::Result<()> {
    let (index, layout) = fields::open_or_create(&index_directory(directory))?;
    let mut writer = index
        .writer_with_num_threads(1, INDEX_HEAP_BYTES)
        .map_err(search_error)?;
    for entry in &ENTRIES {
        writer
            .add_document(writer::document(layout, &indexed(entry)?))
            .map_err(search_error)?;
    }
    writer.commit().map_err(search_error)?;
    writer.wait_merging_threads().map_err(search_error)?;
    let copies = 1;
    Manifest {
        format: FORMAT,
        built_at: BUILT_AT.to_owned(),
        // The collector's database also holds the folded copy.
        documents: (ENTRIES.len() + copies) as u64,
        copies: copies as u64,
        translation_groups: 1,
        last_retrieved_at: Some(LAST_RETRIEVED_AT.to_owned()),
    }
    .write(directory)?;
    Ok(())
}

/// Rejects a symlink or non-directory at `path`; a missing path is fine.
fn plain_directory(path: &Path) -> anyhow::Result<bool> {
    match path.symlink_metadata() {
        Ok(metadata) => {
            anyhow::ensure!(
                metadata.is_dir() && !metadata.file_type().is_symlink(),
                "{} must be a plain directory",
                path.display()
            );
            Ok(true)
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error).with_context(|| format!("cannot inspect {}", path.display())),
    }
}

/// Replaces the snapshot below the disposable runtime directory. Idempotent: a
/// repeated call restores the same corpus, whatever an earlier campaign stage
/// or reload left behind.
pub fn write(runtime: &Path) -> anyhow::Result<()> {
    let directory = runtime.join(SNAPSHOT_PATH);
    let parent = directory
        .parent()
        .ok_or_else(|| anyhow::anyhow!("snapshot directory has no parent"))?;
    if !plain_directory(parent)? {
        fs::create_dir_all(parent)?;
    }
    anyhow::ensure!(
        parent.canonicalize()?.starts_with(runtime.canonicalize()?),
        "KCNA archive fixture must stay inside the runtime directory"
    );
    if plain_directory(&directory)? {
        fs::remove_dir_all(&directory)?;
    }
    fs::create_dir(&directory)?;
    build(&directory)
}

#[cfg(test)]
#[path = "kcna_tests.rs"]
mod tests;
