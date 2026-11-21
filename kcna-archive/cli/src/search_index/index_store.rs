//! Opening, committing, and pruning the index inside a snapshot directory.

use std::{collections::HashSet, path::Path};

use kcna_archive_core::{
    error::search_error,
    search::{
        fields::{self, Fields, ID},
        writer::id_term,
    },
};
use tantivy::{Index, IndexWriter, collector::DocSetCollector, query::AllQuery};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::{Result, io_error};

/// Opens an existing index that can be updated in place.
pub(super) fn open_current(directory: &Path) -> Option<(Index, Fields, Option<OffsetDateTime>)> {
    if !directory.join("meta.json").is_file() {
        return None;
    }
    let (index, fields) = fields::open_or_create(directory).ok()?;
    let payload = index.load_metas().ok()?.payload?;
    let watermark = payload
        .parse::<i128>()
        .ok()
        .and_then(|nanos| OffsetDateTime::from_unix_timestamp_nanos(nanos).ok())?;
    Some((index, fields, Some(watermark)))
}

/// Starts an empty index, discarding one that cannot be reused.
pub(super) fn recreate(directory: &Path) -> Result<(Index, Fields)> {
    match std::fs::remove_dir_all(directory) {
        Ok(()) => {}
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {}
        Err(source) => return Err(io_error(directory)(source)),
    }
    Ok(fields::open_or_create(directory)?)
}

/// Commits with the watermark as payload. Blocking, so it runs off the runtime.
pub(super) async fn commit(
    mut writer: IndexWriter,
    watermark: Option<OffsetDateTime>,
) -> Result<IndexWriter> {
    tokio::task::spawn_blocking(move || -> Result<IndexWriter> {
        let mut prepared = writer.prepare_commit().map_err(search_error)?;
        if let Some(watermark) = watermark {
            prepared.set_payload(&watermark.unix_timestamp_nanos().to_string());
        }
        prepared.commit().map_err(search_error)?;
        Ok(writer)
    })
    .await?
}

pub(super) fn document_count(index: &Index) -> Result<u64> {
    let reader = index.reader().map_err(search_error)?;
    Ok(reader.searcher().num_docs())
}

/// Deletes indexed documents whose identifier the database no longer holds.
pub(super) fn remove_unknown(
    index: &Index,
    fields: Fields,
    writer: &IndexWriter,
    known: &HashSet<Uuid>,
) -> Result<u64> {
    let reader = index.reader().map_err(search_error)?;
    let searcher = reader.searcher();
    let addresses = searcher
        .search(&AllQuery, &DocSetCollector)
        .map_err(search_error)?;
    let mut removed = 0_u64;
    for address in addresses {
        let segment = searcher.segment_reader(address.segment_ord);
        let Some(column) = segment.fast_fields().bytes(ID).map_err(search_error)? else {
            continue;
        };
        let mut bytes = Vec::with_capacity(16);
        for ordinal in column.term_ords(address.doc_id) {
            bytes.clear();
            if column.ord_to_bytes(ordinal, &mut bytes).is_err() {
                continue;
            }
            if let Ok(id) = Uuid::from_slice(&bytes)
                && !known.contains(&id)
            {
                writer.delete_term(id_term(fields, id));
                removed += 1;
            }
        }
    }
    Ok(removed)
}
