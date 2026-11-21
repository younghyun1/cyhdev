//! Builds and refreshes the snapshot from PostgreSQL.

use std::{collections::HashSet, path::Path, time::Instant};

use kcna_archive_core::{
    error::search_error,
    search::{
        engine::SearchEngine,
        fields::Fields,
        writer::{self, IndexedDocument, id_term},
    },
    snapshot::{
        layout::index_directory,
        manifest::{FORMAT, Manifest},
    },
};
use tantivy::{IndexWriter, ReloadPolicy};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use tracing::info;
use uuid::Uuid;

use crate::{
    database::pool::PgPool,
    error::{Error, Result},
    search_index::{
        corpus_stats,
        index_rows::{self, IndexRow, PAGE_ROWS, Rows},
        index_store::{commit, document_count, open_current, recreate, remove_unknown},
    },
};

/// Indexing heap shared by the writer threads.
const WRITER_MEMORY_BYTES: usize = 512_000_000;
const MAX_WRITER_THREADS: usize = 8;

/// Outcome of one snapshot run.
#[derive(Clone, Copy, Debug)]
pub struct IndexReport {
    /// Documents written in this run.
    pub written: u64,
    /// Stale documents removed because the database no longer has them.
    pub removed: u64,
    /// Documents in the index afterwards.
    pub total: u64,
    pub rebuilt: bool,
    pub seconds: f64,
}

/// Brings the snapshot in `snapshot` up to date with the database.
///
/// The commit payload records the newest `document_updated_at` indexed, so a
/// later run reads only rows changed since. An index that is missing, was
/// written with another layout or other storage settings, or carries no
/// payload is rebuilt in full. The manifest is rewritten on every run.
pub async fn sync(pool: &PgPool, snapshot: &Path, rebuild: bool) -> Result<IndexReport> {
    let started = Instant::now();
    let directory = index_directory(snapshot);
    let opened = if rebuild {
        None
    } else {
        open_current(&directory)
    };
    let (index, fields, watermark) = match opened {
        Some(opened) => opened,
        None => {
            let (index, fields) = recreate(&directory)?;
            (index, fields, None)
        }
    };
    let rebuilt = watermark.is_none();
    let threads = std::thread::available_parallelism()
        .map_or(2, std::num::NonZero::get)
        .clamp(1, MAX_WRITER_THREADS);
    let index_writer: IndexWriter = index
        .writer_with_num_threads(threads, WRITER_MEMORY_BYTES)
        .map_err(search_error)?;
    // The entries as they stood before this run. A changed row can alter an
    // entry other than its own: the preferred document it is, or was, a copy
    // of lists it among its sources.
    let previous = if rebuilt {
        index_writer.delete_all_documents().map_err(search_error)?;
        None
    } else {
        Some(SearchEngine::from_index(
            &index,
            fields,
            ReloadPolicy::Manual,
        )?)
    };
    let mut run = Run {
        pool,
        writer: &index_writer,
        fields,
        written: 0,
        indexed: HashSet::new(),
    };
    let mut newest = watermark;
    let mut after: Option<Uuid> = None;
    let mut affected: HashSet<Uuid> = HashSet::new();
    loop {
        let rows = index_rows::load(
            pool,
            Rows::Page {
                after,
                changed_after: watermark,
            },
        )
        .await?;
        let Some(last) = rows.last() else { break };
        after = Some(last.id);
        for row in &rows {
            newest = Some(newest.map_or(row.updated_at, |current| current.max(row.updated_at)));
            let Some(previous) = &previous else { continue };
            index_writer.delete_term(id_term(fields, row.id));
            match row.canonical {
                Some(canonical) => {
                    affected.insert(canonical);
                }
                // A row that stopped being a copy still has its former
                // preferred document standing for it.
                None => {
                    if let Some(entry) = previous.document(row.id)?
                        && entry.id != row.id
                    {
                        affected.insert(entry.id);
                    }
                }
            }
        }
        run.write(rows).await?;
    }
    affected.retain(|id| !run.indexed.contains(id));
    let affected = affected.into_iter().collect::<Vec<_>>();
    for ids in affected.chunks(usize::try_from(PAGE_ROWS).unwrap_or(1_000)) {
        let rows = index_rows::load(pool, Rows::Ids(ids)).await?;
        for row in &rows {
            index_writer.delete_term(id_term(fields, row.id));
        }
        run.write(rows).await?;
    }
    let written = run.written;
    let stored = index_rows::count(pool).await?;
    let mut index_writer = commit(index_writer, newest).await?;
    let mut removed = 0_u64;
    // More indexed than stored means the database dropped documents, which
    // happens when a re-dated document is replaced under a new identifier.
    if document_count(&index)? > stored {
        let known = index_rows::identifiers(pool).await?;
        removed = remove_unknown(&index, fields, &index_writer, &known)?;
        index_writer = commit(index_writer, newest).await?;
    }
    tokio::task::spawn_blocking(move || index_writer.wait_merging_threads().map_err(search_error))
        .await??;
    write_manifest(pool, snapshot).await?;
    Ok(IndexReport {
        written,
        removed,
        total: document_count(&index)?,
        rebuilt,
        seconds: started.elapsed().as_secs_f64(),
    })
}

/// Runs [`sync`] and logs the outcome.
pub async fn sync_and_report(pool: &PgPool, snapshot: &Path, rebuild: bool) -> Result<()> {
    let report = sync(pool, snapshot, rebuild).await?;
    info!(
        written = report.written,
        removed = report.removed,
        total = report.total,
        rebuilt = report.rebuilt,
        seconds = report.seconds,
        snapshot = %snapshot.display(),
        "Snapshot is up to date"
    );
    Ok(())
}

/// The index writer and what one run has written through it.
struct Run<'a> {
    pool: &'a PgPool,
    writer: &'a IndexWriter,
    fields: Fields,
    written: u64,
    /// Identifiers written in this run, so an entry is not written twice.
    indexed: HashSet<Uuid>,
}

impl Run<'_> {
    /// Writes the preferred documents among `rows`. A copy is represented by
    /// its preferred document's entry and is skipped here.
    async fn write(&mut self, rows: Vec<IndexRow>) -> Result<()> {
        let preferred = rows
            .iter()
            .filter(|row| row.canonical.is_none())
            .map(|row| row.id)
            .collect::<Vec<_>>();
        let mut copies = index_rows::copies(self.pool, &preferred).await?;
        for row in rows {
            if row.canonical.is_some() || !self.indexed.insert(row.id) {
                continue;
            }
            let links = copies.remove(&row.id).unwrap_or_default();
            // Only one of the copies may carry the translation group.
            let translation_group = row
                .translation_group
                .or_else(|| links.iter().find_map(|link| link.translation_group));
            let indexed = IndexedDocument {
                id: row.id,
                title: row.title,
                body: row.body,
                date: row.date,
                precision: row.precision,
                language: row.language,
                source: row.source,
                canonical_url: row.canonical_url,
                category: row.category,
                source_record_id: row.source_record_id,
                metadata: row.metadata,
                translation_group,
                copies: links.into_iter().map(|link| link.copy).collect(),
            };
            self.writer
                .add_document(writer::document(self.fields, &indexed))
                .map_err(search_error)?;
            self.written += 1;
            if self.written.is_multiple_of(50_000) {
                info!(written = self.written, "Snapshot progress");
            }
        }
        Ok(())
    }
}

/// Records what the database knows beyond the index, for the status page.
async fn write_manifest(pool: &PgPool, snapshot: &Path) -> Result<()> {
    let stats = corpus_stats::read(pool).await?;
    let format_time = |time: OffsetDateTime| {
        time.format(&Rfc3339)
            .map_err(|error| Error::InvalidRecord(error.to_string()))
    };
    let count = |value: i64| u64::try_from(value).unwrap_or(0);
    let manifest = Manifest {
        format: FORMAT,
        built_at: format_time(OffsetDateTime::now_utc())?,
        documents: count(stats.documents),
        copies: count(stats.copies),
        translation_groups: count(stats.translation_groups),
        last_retrieved_at: match stats.last_retrieved_at {
            Some(time) => Some(format_time(time)?),
            None => None,
        },
    };
    Ok(manifest.write(snapshot)?)
}
