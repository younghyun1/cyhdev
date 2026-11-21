//! Loads a snapshot's index from disk into process memory.
//!
//! The website answers every archive request from memory: no file is read and
//! no database is asked once the load finishes. The stored text stays
//! Zstandard-compressed in memory and is decompressed one block per document
//! read, so the resident size is the size of the index on disk.

use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use memmap2::Mmap;
use tantivy::{
    Index,
    directory::{Directory, RamDirectory, TerminatingWrite},
};

use crate::error::{Error, Result, io_error, search_error};

/// An index is a few dozen files per segment; more means the directory is not
/// a snapshot.
const MAX_INDEX_FILES: usize = 4_096;
/// Refuses to read more than this into memory. The corpus index is under one
/// gibibyte; the bound stops a wrong path from exhausting the host.
const MAX_INDEX_BYTES: u64 = 8 * 1024 * 1024 * 1024;
/// Lock files belong to the process that wrote the index.
const LOCK_PREFIX: &str = ".tantivy-";

/// An index held in memory, and how many bytes it occupies.
pub struct LoadedIndex {
    pub index: Index,
    pub bytes: u64,
    pub files: usize,
}

/// Copies one file into the in-memory directory.
///
/// The file is mapped and written in a single call, so the directory
/// allocates its buffer once at the final size and the only copy made is page
/// cache to heap. Reading the file into a buffer first would hold every byte
/// twice on the heap for the largest file, which is most of the index.
fn copy_into(memory: &RamDirectory, source: &Path, name: &Path, length: u64) -> Result<()> {
    let mut writer = memory
        .open_write(name)
        .map_err(|error| Error::Search(error.to_string()))?;
    // A zero-length file cannot be mapped and needs no bytes written.
    if length > 0 {
        let file = File::open(source).map_err(io_error(source))?;
        // SAFETY: the mapping is read-only and dropped before this function
        // returns. Snapshot files are immutable once written: the snapshot
        // writer only adds files and a new snapshot replaces a file by rename,
        // which leaves this mapping on the old inode. Truncating a file in
        // place while a server loads it is outside the snapshot contract.
        let mapped = unsafe { Mmap::map(&file) }.map_err(io_error(source))?;
        writer.write_all(&mapped).map_err(io_error(source))?;
    }
    writer.terminate().map_err(io_error(source))
}

/// Reads every file of the index in `directory` into memory and opens it.
pub fn load(directory: &Path) -> Result<LoadedIndex> {
    let entries = fs::read_dir(directory).map_err(io_error(directory))?;
    let memory = RamDirectory::create();
    let mut bytes = 0_u64;
    let mut files = 0_usize;
    for entry in entries {
        let entry = entry.map_err(io_error(directory))?;
        let path = entry.path();
        let metadata = entry.metadata().map_err(io_error(&path))?;
        if !metadata.is_file() {
            continue;
        }
        let name = entry.file_name();
        if name.to_string_lossy().starts_with(LOCK_PREFIX) {
            continue;
        }
        files += 1;
        bytes = bytes.saturating_add(metadata.len());
        if files > MAX_INDEX_FILES || bytes > MAX_INDEX_BYTES {
            return Err(Error::Snapshot(format!(
                "{} holds more than {MAX_INDEX_FILES} files or {MAX_INDEX_BYTES} bytes",
                directory.display()
            )));
        }
        copy_into(&memory, &path, Path::new(&name), metadata.len())?;
    }
    if files == 0 {
        return Err(Error::Snapshot(format!(
            "{} holds no index files",
            directory.display()
        )));
    }
    let index = Index::open(memory).map_err(search_error)?;
    Ok(LoadedIndex {
        index,
        bytes,
        files,
    })
}
