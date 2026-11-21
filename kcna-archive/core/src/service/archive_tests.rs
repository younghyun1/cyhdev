//! Tests for the snapshot round trip: write to disk, load into memory, serve.

use tantivy::{Index, IndexWriter};

use crate::{
    error::{Error, Result, io_error, search_error},
    search::{engine_tests::corpus, fields, writer},
    service::{
        archive::{Archive, ServiceError},
        export::ExportFormat,
        params::QueryParams,
    },
    snapshot::{
        layout::index_directory,
        manifest::{FORMAT, Manifest},
    },
};

fn write_snapshot(directory: &std::path::Path) -> Result<()> {
    let (index, layout): (Index, _) = fields::open_or_create(&index_directory(directory))?;
    let mut index_writer: IndexWriter = index
        .writer_with_num_threads(1, 20_000_000)
        .map_err(search_error)?;
    for document in corpus() {
        index_writer
            .add_document(writer::document(layout, &document))
            .map_err(search_error)?;
    }
    index_writer.commit().map_err(search_error)?;
    index_writer.wait_merging_threads().map_err(search_error)?;
    Manifest {
        format: FORMAT,
        built_at: "2026-11-21T12:00:00Z".to_owned(),
        documents: 7,
        copies: 1,
        translation_groups: 2,
        last_retrieved_at: Some("2026-11-21T11:00:00Z".to_owned()),
    }
    .write(directory)
}

fn query(text: &str) -> QueryParams {
    QueryParams {
        q: text.to_owned(),
        ..QueryParams::default()
    }
}

fn service<T>(result: std::result::Result<T, ServiceError>) -> Result<T> {
    result.map_err(|error| Error::Search(error.to_string()))
}

#[test]
fn a_snapshot_loaded_into_memory_serves_without_its_files() -> Result<()> {
    let directory = tempfile::tempdir().map_err(io_error("tempdir"))?;
    write_snapshot(directory.path())?;
    let archive = Archive::load_into_memory(directory.path())?;
    // Nothing may be read from disk after the load.
    directory.close().map_err(io_error("tempdir"))?;

    assert!(archive.memory_bytes().is_some_and(|bytes| bytes > 0));
    let found = service(archive.search(&query("김정은")))?;
    assert_eq!(found.total, 1);
    assert!(found.items.first().is_some_and(|item| {
        item.title.iter().any(|segment| segment.hit) && !item.snippet.is_empty()
    }));

    let status = service(archive.status())?;
    assert_eq!((status.documents, status.copies, status.indexed), (7, 1, 6));
    assert_eq!(status.oldest_date.as_deref(), Some("2003-05-31"));
    assert_eq!(status.newest_date.as_deref(), Some("2026-11-20"));
    assert_eq!(status.snapshot_built_at, "2026-11-21T12:00:00Z");

    let id = found.items.first().map(|item| item.id.clone());
    let Some(id) = id else {
        return Err(Error::Search("no result to open".to_owned()));
    };
    let document = service(archive.document(&id, "핵"))?;
    assert!(
        document
            .paragraphs
            .iter()
            .flatten()
            .any(|segment| segment.hit)
    );
    assert!(matches!(
        archive.document("not-a-uuid", ""),
        Err(ServiceError::BadRequest)
    ));
    assert!(matches!(
        archive.document("00000000-0000-0000-0000-0000000003e7", ""),
        Err(ServiceError::NotFound)
    ));

    let csv = service(archive.export(&query("peace"), ExportFormat::Csv))?;
    assert_eq!(csv.lines().count(), 3);
    let trends = service(archive.trends(&QueryParams::default(), vec!["peace".to_owned()]))?;
    assert_eq!(trends.starts.len(), trends.totals.len());
    assert!(matches!(
        archive.trends(&QueryParams::default(), Vec::new()),
        Err(ServiceError::BadRequest)
    ));
    Ok(())
}

#[test]
fn a_directory_without_a_snapshot_is_refused() -> Result<()> {
    let directory = tempfile::tempdir().map_err(io_error("tempdir"))?;
    assert!(Archive::load_into_memory(directory.path()).is_err());
    assert!(Archive::open_on_disk(directory.path()).is_err());
    Ok(())
}
