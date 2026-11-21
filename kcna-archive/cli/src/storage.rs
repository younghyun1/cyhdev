//! Atomic raw-page persistence with URL-to-path validation.

use std::{
    ffi::OsString,
    path::{Component, Path, PathBuf},
};

use sha2::{Digest, Sha256};
use tokio::fs;
use url::Url;

use crate::error::{Result, io_error};

/// Maps a source URL to a safe path below the supplied root.
pub fn url_path(root: &Path, url: &Url) -> PathBuf {
    let mut output = root.to_path_buf();
    for component in Path::new(url.path()).components() {
        if let Component::Normal(part) = component {
            output.push(part);
        }
    }
    if url.path().ends_with('/') || output == root {
        output.push("index.html");
    }
    output
}

/// Returns the raw bytes when a previously collected page exists.
pub async fn read_if_exists(path: &Path) -> Result<Option<Vec<u8>>> {
    match fs::read(path).await {
        Ok(bytes) => Ok(Some(bytes)),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(io_error(path)(source)),
    }
}

/// Reports whether a collected page exists without reading its bytes.
pub async fn exists(path: &Path) -> Result<bool> {
    fs::try_exists(path).await.map_err(io_error(path))
}

/// Writes bytes through a sibling temporary file, then atomically renames them.
pub async fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).await.map_err(io_error(parent))?;
    let temporary = temporary_path(path);
    fs::write(&temporary, bytes)
        .await
        .map_err(io_error(&temporary))?;
    fs::rename(&temporary, path).await.map_err(io_error(path))?;
    Ok(())
}

/// Appends the suffix instead of replacing the extension, so `a.htm` and
/// `a.html` downloading concurrently never share one temporary file.
fn temporary_path(path: &Path) -> PathBuf {
    let mut value: OsString = path.as_os_str().to_owned();
    value.push(".part");
    PathBuf::from(value)
}

/// Produces a stable content digest for the retrieval manifest.
pub fn sha256(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_urls_below_the_root() -> Result<()> {
        let root = Path::new("data/raw/kcna_co_jp");
        let cases = [
            (
                "http://www.kcna.co.jp/item/2002/a.htm",
                "data/raw/kcna_co_jp/item/2002/a.htm",
            ),
            ("http://www.kcna.co.jp/", "data/raw/kcna_co_jp/index.html"),
            (
                "http://www.kcna.co.jp/item/",
                "data/raw/kcna_co_jp/item/index.html",
            ),
            (
                "http://www.kcna.co.jp/a/../../b.htm?x=1#y",
                "data/raw/kcna_co_jp/b.htm",
            ),
        ];
        for (url, expected) in cases {
            assert_eq!(url_path(root, &Url::parse(url)?), Path::new(expected));
        }
        Ok(())
    }

    #[test]
    fn temporary_names_keep_the_original_extension() {
        assert_ne!(
            temporary_path(Path::new("a/b.htm")),
            temporary_path(Path::new("a/b.html"))
        );
        assert_eq!(
            temporary_path(Path::new("a/b.htm")),
            Path::new("a/b.htm.part")
        );
    }
}
