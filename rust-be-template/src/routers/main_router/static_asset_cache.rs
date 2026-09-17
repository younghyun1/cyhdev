//! Browser cache policy for the finite asset inventory embedded by the frontend build.

use std::collections::{HashMap, HashSet};

use rust_embed::Embed;
use serde::Deserialize;

pub(super) const REVALIDATE: &str = "public, max-age=0, must-revalidate";
pub(super) const IMMUTABLE: &str = "public, max-age=31536000, immutable";

#[derive(Deserialize)]
struct ManifestEntry {
    file: String,
    #[serde(default)]
    css: Vec<String>,
    #[serde(default)]
    assets: Vec<String>,
}

/// Fail conservatively to revalidation for older builds or a malformed manifest.
pub(super) fn immutable_paths<A: Embed>() -> HashSet<String> {
    let Some(manifest) = A::get(".vite/manifest.json") else {
        return HashSet::new();
    };
    let entries: HashMap<String, ManifestEntry> = match serde_json::from_slice(&manifest.data) {
        Ok(entries) => entries,
        Err(error) => {
            tracing::warn!(error = %error, "Invalid frontend manifest; assets will revalidate");
            return HashSet::new();
        }
    };
    entries
        .into_values()
        .flat_map(|entry| {
            std::iter::once(entry.file)
                .chain(entry.css)
                .chain(entry.assets)
        })
        .filter(|path| fingerprinted(path))
        .collect()
}

/// Require Vite's default hashed filename shape as well as manifest membership.
fn fingerprinted(path: &str) -> bool {
    let Some(name) = path.strip_prefix("assets/") else {
        return false;
    };
    let Some((stem, extension)) = name.rsplit_once('.') else {
        return false;
    };
    if name.contains('/')
        || !matches!(
            extension,
            "js" | "css"
                | "woff"
                | "woff2"
                | "ttf"
                | "otf"
                | "png"
                | "jpg"
                | "jpeg"
                | "webp"
                | "avif"
                | "svg"
                | "gif"
                | "ico"
                | "wasm"
        )
    {
        return false;
    }
    // Hashes can themselves contain '-' and '_', so inspect the fixed-width suffix.
    let bytes = stem.as_bytes();
    bytes.len() > 9
        && bytes[bytes.len() - 9] == b'-'
        && bytes[bytes.len() - 8..]
            .iter()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-'))
}
