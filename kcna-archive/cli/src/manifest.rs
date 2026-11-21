//! Append-only retrieval evidence for each newly stored response.

use std::path::{Path, PathBuf};

use jiff::Timestamp;
use serde::Serialize;
use tokio::{
    fs::{self, File, OpenOptions},
    io::{AsyncWriteExt, BufWriter},
    sync::Mutex,
};
use url::Url;

use crate::{
    error::{Result, io_error},
    storage,
};

/// Records buffered before a flush. Kept small because anything still buffered
/// when the process is killed leaves a raw file without retrieval evidence.
const FLUSH_INTERVAL: usize = 32;

/// Serializes manifest writes without buffering an unbounded in-memory catalog.
pub struct Manifest {
    data_dir: PathBuf,
    path: PathBuf,
    state: Mutex<WriterState>,
}

struct WriterState {
    writer: BufWriter<File>,
    pending: usize,
}

#[derive(Debug, Serialize)]
struct Record<'a> {
    retrieved_at: String,
    source: &'a str,
    url: &'a str,
    path: String,
    bytes: usize,
    sha256: String,
}

impl Manifest {
    /// Opens the durable append-only manifest location.
    pub async fn open(data_dir: &Path) -> Result<Self> {
        let directory = data_dir.join("manifests");
        fs::create_dir_all(&directory)
            .await
            .map_err(io_error(&directory))?;
        let path = directory.join("retrievals.jsonl");
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .await
            .map_err(io_error(&path))?;
        Ok(Self {
            data_dir: data_dir.to_owned(),
            path,
            state: Mutex::new(WriterState {
                writer: BufWriter::with_capacity(64 * 1024, file),
                pending: 0,
            }),
        })
    }

    /// Records a completed atomic write.
    pub async fn record(&self, source: &str, url: &Url, path: &Path, bytes: &[u8]) -> Result<()> {
        let record = Record {
            retrieved_at: Timestamp::now().to_string(),
            source,
            url: url.as_str(),
            path: manifest_path(&self.data_dir, path),
            bytes: bytes.len(),
            sha256: storage::sha256(bytes),
        };
        let mut encoded = serde_json::to_vec(&record)?;
        encoded.push(b'\n');
        let mut state = self.state.lock().await;
        state
            .writer
            .write_all(&encoded)
            .await
            .map_err(io_error(&self.path))?;
        state.pending += 1;
        if state.pending >= FLUSH_INTERVAL {
            state.writer.flush().await.map_err(io_error(&self.path))?;
            state.pending = 0;
        }
        Ok(())
    }

    /// Flushes retrieval evidence; called on every exit path of a collection run.
    pub async fn flush(&self) -> Result<()> {
        let mut state = self.state.lock().await;
        state.writer.flush().await.map_err(io_error(&self.path))?;
        state.pending = 0;
        Ok(())
    }
}

/// Stores paths relative to the data directory so the manifest stays valid
/// when the corpus moves or `--data-dir` is spelled differently.
fn manifest_path(data_dir: &Path, path: &Path) -> String {
    path.strip_prefix(data_dir)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}

/// Normalizes a recorded or current path to its `raw/...` suffix.
///
/// Earlier manifests recorded paths relative to the working directory
/// (`data/raw/...`); matching on the suffix keeps both forms resolvable.
pub fn corpus_key(path: &Path) -> String {
    let mut components = path.components();
    let mut key = PathBuf::new();
    for component in components.by_ref() {
        if component.as_os_str() == "raw" {
            key.push(component);
            break;
        }
    }
    if key.as_os_str().is_empty() {
        return path.to_string_lossy().into_owned();
    }
    key.extend(components);
    key.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_paths_relative_to_the_data_directory() {
        assert_eq!(
            manifest_path(Path::new("/srv/kcna"), Path::new("/srv/kcna/raw/a/b.htm")),
            "raw/a/b.htm"
        );
    }

    #[test]
    fn corpus_keys_ignore_the_data_directory_spelling() {
        let expected = "raw/kcna_co_jp/item/a.htm";
        for path in [
            "data/raw/kcna_co_jp/item/a.htm",
            "./data/raw/kcna_co_jp/item/a.htm",
            "/srv/kcna/raw/kcna_co_jp/item/a.htm",
            "raw/kcna_co_jp/item/a.htm",
        ] {
            assert_eq!(corpus_key(Path::new(path)), expected);
        }
        assert_eq!(corpus_key(Path::new("elsewhere/a.htm")), "elsewhere/a.htm");
    }
}
