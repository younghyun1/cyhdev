//! Builds a tiny snapshot in a temporary directory for tests.

use std::{fs, path::Path};

use kcna_archive_core::{
    error::{Error as CoreError, io_error, search_error},
    search::{
        fields,
        writer::{self, IndexedDocument},
    },
    snapshot::{
        layout::{index_directory, manifest_path},
        manifest::{FORMAT, Manifest},
    },
    types::{Language, PublicationPrecision, Source},
};
use tantivy::IndexWriter;
use time::{Date, Month};
use uuid::Uuid;

/// Id of the Korean article in the fixture.
pub(crate) const KOREAN_ARTICLE_ID: &str = "00000000-0000-0000-0000-000000000001";

fn article(
    id: u128,
    title: &str,
    body: &str,
    language: Language,
    (year, month, day): (i32, Month, u8),
) -> Result<IndexedDocument, CoreError> {
    let date = Date::from_calendar_date(year, month, day)
        .map_err(|source| CoreError::Snapshot(source.to_string()))?;
    Ok(IndexedDocument {
        id: Uuid::from_u128(id),
        title: title.to_owned(),
        body: Some(body.to_owned()),
        date,
        precision: PublicationPrecision::Day,
        language,
        source: Source::KcnaKp,
        canonical_url: format!("https://example.invalid/articles/{id}"),
        category: Some("news".to_owned()),
        source_record_id: None,
        metadata: serde_json::Value::Null,
        translation_group: None,
        copies: Vec::new(),
    })
}

/// Writes a three-article snapshot into `directory`, replacing any earlier one
/// so a repeated call restores the fixture instead of appending to it.
pub(crate) fn write_snapshot(directory: &Path) -> Result<(), CoreError> {
    let index_path = index_directory(directory);
    if index_path.exists() {
        fs::remove_dir_all(&index_path).map_err(io_error(&index_path))?;
    }
    let (index, layout) = fields::open_or_create(&index_path)?;
    let mut writer: IndexWriter = index
        .writer_with_num_threads(1, 20_000_000)
        .map_err(search_error)?;
    let articles = [
        article(
            1,
            "김정은 동지",
            "핵 억제력 강화에 관한 보도.\n\n두번째 단락.",
            Language::Korean,
            (2026, Month::November, 20),
        )?,
        article(
            2,
            "Peace talks resume",
            "Delegations discussed peace and trade.",
            Language::English,
            (2025, Month::March, 2),
        )?,
        article(
            3,
            "Peace anniversary",
            "A peace anniversary was marked.",
            Language::English,
            (2024, Month::June, 15),
        )?,
    ];
    for article in &articles {
        writer
            .add_document(writer::document(layout, article))
            .map_err(search_error)?;
    }
    writer.commit().map_err(search_error)?;
    writer.wait_merging_threads().map_err(search_error)?;
    Manifest {
        format: FORMAT,
        built_at: "2026-11-21T12:00:00Z".to_owned(),
        documents: 3,
        copies: 0,
        translation_groups: 0,
        last_retrieved_at: None,
    }
    .write(directory)
}

/// Replaces the manifest with one for an unsupported format, so loading fails.
pub(crate) fn corrupt_manifest(directory: &Path) -> Result<(), CoreError> {
    let path = manifest_path(directory);
    fs::write(
        &path,
        r#"{"format":999,"built_at":"x","documents":0,"copies":0,"translation_groups":0}"#,
    )
    .map_err(io_error(&path))
}
