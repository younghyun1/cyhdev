//! Index layout shared by the writer and the query path.

use std::path::Path;

use tantivy::{
    Index, IndexSettings,
    schema::{
        BytesOptions, FAST, Field, INDEXED, IndexRecordOption, STORED, STRING, Schema,
        TextFieldIndexing, TextOptions,
    },
    store::{Compressor, ZstdCompressor},
};
use time::Date;

use crate::{
    error::{Error, Result, io_error, search_error},
    search::tokenizer::ArchiveTokenizer,
    types::{Language, PublicationPrecision, Source},
};

const TOKENIZER: &str = "kcna";

/// Zstandard level of the stored text. Titles and bodies are most of the
/// index, and the website holds the whole index in memory, so the store is
/// compressed as far as the format goes; reading a block back stays cheap
/// because decompression speed does not depend on the level.
pub const DOCSTORE_ZSTD_LEVEL: i32 = 22;
/// Stored documents are compressed in blocks of this size. Larger blocks
/// compress repeated newsroom phrasing better; a result page decompresses one
/// block per hit, so the block stays small enough to cost microseconds.
const DOCSTORE_BLOCK_BYTES: usize = 64 * 1024;

/// Handles to every field of the index.
#[derive(Clone, Copy, Debug)]
pub struct Fields {
    /// Document UUID bytes; indexed so a changed document can be replaced.
    pub id: Field,
    pub title: Field,
    pub body: Field,
    /// Publication date as a Julian day number: range filters, date sorting,
    /// and histogram bucketing all read this one column.
    pub date: Field,
    pub language: Field,
    pub source: Field,
    pub category: Field,
    /// How exactly the source stated the publication time; stored only.
    pub precision: Field,
    /// Address of the article at its source; stored only.
    pub url: Field,
    /// The source's own article identifier; stored only.
    pub record: Field,
    /// Source metadata as JSON text; stored only.
    pub metadata: Field,
    /// Translation group UUID bytes, indexed to find the other editions.
    pub group: Field,
    /// UUID bytes of the copies this entry stands for, so a link to a copy
    /// opens the preferred document.
    pub alias: Field,
    /// The copies as JSON text, for the reader's source list; stored only.
    pub copies: Field,
}

pub const ID: &str = "id";
pub const DATE: &str = "date";
pub const LANGUAGE: &str = "language";
pub const SOURCE: &str = "source";
pub const CATEGORY: &str = "category";

fn schema() -> (Schema, Fields) {
    // Positions are required for the bigram phrases that match CJK words; the
    // text is stored so results, snippets, and the reader need no database.
    let text = TextOptions::default()
        .set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer(TOKENIZER)
                .set_index_option(IndexRecordOption::WithFreqsAndPositions),
        )
        .set_stored();
    let key = || BytesOptions::default().set_indexed();
    let mut builder = Schema::builder();
    let fields = Fields {
        // Fast as well, so stale documents can be found without reading every
        // stored document back.
        id: builder.add_bytes_field(ID, key().set_stored().set_fast()),
        title: builder.add_text_field("title", text.clone()),
        body: builder.add_text_field("body", text),
        date: builder.add_i64_field(DATE, INDEXED | FAST | STORED),
        language: builder.add_u64_field(LANGUAGE, INDEXED | FAST | STORED),
        source: builder.add_u64_field(SOURCE, INDEXED | FAST | STORED),
        category: builder.add_text_field(CATEGORY, STRING | FAST | STORED),
        precision: builder.add_u64_field("precision", STORED),
        url: builder.add_text_field("url", STORED),
        record: builder.add_text_field("record", STORED),
        metadata: builder.add_text_field("metadata", STORED),
        group: builder.add_bytes_field("group", key().set_stored()),
        alias: builder.add_bytes_field("alias", key()),
        copies: builder.add_text_field("copies", STORED),
    };
    (builder.build(), fields)
}

fn settings() -> IndexSettings {
    IndexSettings {
        docstore_compression: Compressor::Zstd(ZstdCompressor {
            compression_level: Some(DOCSTORE_ZSTD_LEVEL),
        }),
        docstore_blocksize: DOCSTORE_BLOCK_BYTES,
        ..IndexSettings::default()
    }
}

/// Creates an index in memory with the production layout, for tests.
#[cfg(test)]
pub fn create_in_ram() -> Result<(Index, Fields)> {
    let (schema, fields) = schema();
    let index = Index::builder()
        .schema(schema)
        .settings(settings())
        .create_in_ram()
        .map_err(search_error)?;
    index.tokenizers().register(TOKENIZER, ArchiveTokenizer);
    Ok((index, fields))
}

/// Opens the index in `directory` from disk, creating an empty one when absent.
///
/// Fails when the directory holds an index with another layout; the writer
/// treats that as a reason to rebuild.
pub fn open_or_create(directory: &Path) -> Result<(Index, Fields)> {
    std::fs::create_dir_all(directory).map_err(io_error(directory))?;
    let (schema, fields) = schema();
    let mmap = tantivy::directory::MmapDirectory::open(directory)
        .map_err(|error| Error::Search(error.to_string()))?;
    let index = Index::builder()
        .schema(schema)
        .settings(settings())
        .open_or_create(mmap)
        .map_err(search_error)?;
    if !has_current_settings(&index) {
        return Err(Error::Search(
            "index was written with other storage settings".to_owned(),
        ));
    }
    index.tokenizers().register(TOKENIZER, ArchiveTokenizer);
    Ok((index, fields))
}

/// Adopts an index that was opened from another directory implementation,
/// checking that it has the layout this crate reads.
pub fn adopt(index: &Index) -> Result<Fields> {
    let (schema, fields) = schema();
    if index.schema() != schema {
        return Err(Error::Snapshot(
            "index layout differs from this build; rebuild the snapshot".to_owned(),
        ));
    }
    index.tokenizers().register(TOKENIZER, ArchiveTokenizer);
    Ok(fields)
}

/// An index opened from disk keeps the settings it was created with, so one
/// from before the compressed store has to be recognized and replaced.
fn has_current_settings(index: &Index) -> bool {
    let expected = settings();
    let actual = index.settings();
    actual.docstore_compression == expected.docstore_compression
        && actual.docstore_blocksize == expected.docstore_blocksize
}

/// Stable numeric codes; changing one requires rebuilding the index.
pub const fn language_code(language: Language) -> u64 {
    match language {
        Language::Korean => 0,
        Language::English => 1,
        Language::Chinese => 2,
        Language::Russian => 3,
        Language::Spanish => 4,
        Language::Japanese => 5,
    }
}

pub const fn language_from_code(code: u64) -> Option<Language> {
    match code {
        0 => Some(Language::Korean),
        1 => Some(Language::English),
        2 => Some(Language::Chinese),
        3 => Some(Language::Russian),
        4 => Some(Language::Spanish),
        5 => Some(Language::Japanese),
        _ => None,
    }
}

pub const fn source_code(source: Source) -> u64 {
    match source {
        Source::KcnaKp => 0,
        Source::KcnaCoJp => 1,
        Source::StilesArchive => 2,
    }
}

pub const fn source_from_code(code: u64) -> Option<Source> {
    match code {
        0 => Some(Source::KcnaKp),
        1 => Some(Source::KcnaCoJp),
        2 => Some(Source::StilesArchive),
        _ => None,
    }
}

pub const fn precision_code(precision: PublicationPrecision) -> u64 {
    match precision {
        PublicationPrecision::Second => 0,
        PublicationPrecision::Minute => 1,
        PublicationPrecision::Day => 2,
        PublicationPrecision::InferredDay => 3,
    }
}

pub const fn precision_from_code(code: u64) -> Option<PublicationPrecision> {
    match code {
        0 => Some(PublicationPrecision::Second),
        1 => Some(PublicationPrecision::Minute),
        2 => Some(PublicationPrecision::Day),
        3 => Some(PublicationPrecision::InferredDay),
        _ => None,
    }
}

pub fn date_value(date: Date) -> i64 {
    i64::from(date.to_julian_day())
}

pub fn date_from_value(value: i64) -> Option<Date> {
    i32::try_from(value)
        .ok()
        .and_then(|day| Date::from_julian_day(day).ok())
}

#[cfg(test)]
mod tests {
    use time::macros::date;

    use super::*;

    #[test]
    fn codes_round_trip() {
        for language in [
            Language::Korean,
            Language::English,
            Language::Chinese,
            Language::Russian,
            Language::Spanish,
            Language::Japanese,
        ] {
            assert_eq!(language_from_code(language_code(language)), Some(language));
        }
        for source in [Source::KcnaKp, Source::KcnaCoJp, Source::StilesArchive] {
            assert_eq!(source_from_code(source_code(source)), Some(source));
        }
        for precision in PublicationPrecision::ALL {
            assert_eq!(
                precision_from_code(precision_code(precision)),
                Some(precision)
            );
        }
        assert_eq!(language_from_code(99), None);
        let day = date!(2026 - 11 - 21);
        assert_eq!(date_from_value(date_value(day)), Some(day));
        assert!(date_value(date!(2026 - 11 - 22)) == date_value(day) + 1);
    }
}
