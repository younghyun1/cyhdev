//! Collector for KCNA's current multilingual archive and POST pagination.

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::Arc,
};

use futures::{StreamExt, stream};
use tokio::time::{Duration, sleep};
use tracing::{info, warn};
use url::Url;

use crate::{
    archive::{Summary, live_lists},
    cli::LiveEdition,
    error::{Error, Result, io_error},
    fetch::Fetcher,
    html,
    importer::extract,
    manifest::Manifest,
    storage,
};

const BASE: &str = "http://www.kcna.kp";
pub(super) const SOURCE: &str = "kcna.kp";
const MAX_DETAILS_PER_LANGUAGE: usize = 250_000;
const SEMANTIC_ATTEMPTS: usize = 4;
const ARTICLE_WINDOW: usize = 256;

/// Shared read-only context for one language edition.
pub(super) struct Edition<'a> {
    pub(super) data_dir: &'a Path,
    pub(super) manifest: &'a Manifest,
    pub(super) fetcher: &'a Fetcher,
    pub(super) edition: LiveEdition,
    /// Article identifiers already stored, captured before enumeration.
    pub(super) known: HashSet<String>,
}

/// Enumerates all language-specific list pages, then fetches every linked article.
pub async fn run(
    data_dir: &Path,
    manifest: Arc<Manifest>,
    fetcher: Arc<Fetcher>,
    editions: Vec<LiveEdition>,
) -> Result<Summary> {
    let mut requested = HashSet::with_capacity(editions.len());
    let unique = editions
        .into_iter()
        .filter(|edition| requested.insert(*edition));
    let results = stream::iter(unique)
        .map(|edition| {
            let manifest = manifest.as_ref();
            let fetcher = fetcher.as_ref();
            async move {
                (
                    edition,
                    collect_edition(data_dir, manifest, fetcher, edition).await,
                )
            }
        })
        .buffer_unordered(2)
        .collect::<Vec<_>>()
        .await;
    let mut summary = Summary::default();
    for (edition, result) in results {
        match result {
            Ok(edition_summary) => summary = summary + edition_summary,
            Err(error) => {
                // One edition failing must not discard the others' totals.
                summary.failed += 1;
                warn!(language = edition.path(), error = %error, "Live edition failed");
            }
        }
    }
    Ok(summary)
}

async fn collect_edition(
    data_dir: &Path,
    manifest: &Manifest,
    fetcher: &Fetcher,
    edition: LiveEdition,
) -> Result<Summary> {
    let language = edition.path();
    let root_url = Url::parse(&format!("{BASE}/{language}"))?;
    let homepage = fetcher.get(&root_url).await?;
    let categories = article_links(&root_url, &html::decode(&homepage), "/article/list/")?;
    let context = Edition {
        data_dir,
        manifest,
        fetcher,
        edition,
        known: stored_article_ids(&article_directory(data_dir, edition)).await?,
    };
    info!(
        language,
        categories = categories.len(),
        stored_articles = context.known.len(),
        "Enumerating live archive categories"
    );
    let mut detail_urls = HashSet::new();
    let mut summary = Summary::default();
    for category in categories {
        match live_lists::enumerate_category(&context, &category).await {
            Ok((links, category_summary)) => {
                summary = summary + category_summary;
                detail_urls.extend(links);
            }
            Err(error) => {
                summary.failed += 1;
                warn!(language, category = %category, error = %error, "Live category failed");
            }
        }
        if detail_urls.len() > MAX_DETAILS_PER_LANGUAGE {
            return Err(Error::CrawlLimit {
                limit: MAX_DETAILS_PER_LANGUAGE,
            });
        }
    }
    info!(
        language,
        articles = detail_urls.len(),
        "Downloading live archive articles"
    );
    let results = stream::iter(detail_urls)
        .map(|url| collect_article(&context, url))
        .buffer_unordered(ARTICLE_WINDOW)
        .collect::<Vec<_>>()
        .await;
    for result in results {
        match result {
            Ok(true) => summary.downloaded += 1,
            Ok(false) => summary.reused += 1,
            Err(error) => {
                // An article can leave the site's rolling window between
                // enumeration and download.
                summary.record_error(&error);
                warn!(language, error = %error, "Live article failed");
            }
        }
    }
    Ok(summary)
}

async fn collect_article(context: &Edition<'_>, url: Url) -> Result<bool> {
    let article_id = article_id(&url).ok_or(Error::MissingField("article id"))?;
    let path =
        article_directory(context.data_dir, context.edition).join(format!("{article_id}.html"));
    if let Some(bytes) = storage::read_if_exists(&path).await? {
        if extract::live(&html::decode(&bytes)).is_ok() {
            return Ok(false);
        }
        warn!(url = %url, path = %path.display(), "Replacing non-article response");
    }
    let bytes = fetch_valid_article(context.fetcher, &url).await?;
    storage::write_atomic(&path, &bytes).await?;
    context.manifest.record(SOURCE, &url, &path, &bytes).await?;
    Ok(true)
}

async fn fetch_valid_article(fetcher: &Fetcher, url: &Url) -> Result<Vec<u8>> {
    for attempt in 1..=SEMANTIC_ATTEMPTS {
        let bytes = fetcher.get(url).await?;
        if extract::live(&html::decode(&bytes)).is_ok() {
            return Ok(bytes);
        }
        warn!(url = %url, attempt, "Retrying HTTP 200 response without article content");
        sleep(Duration::from_millis(500 * attempt as u64)).await;
    }
    Err(Error::InvalidRecord(format!(
        "article response remained invalid for {url}"
    )))
}

fn article_directory(data_dir: &Path, edition: LiveEdition) -> PathBuf {
    data_dir.join(format!("raw/kcna_kp/articles/{}", edition.path()))
}

/// Lists stored article identifiers; a missing directory is an empty corpus.
async fn stored_article_ids(directory: &Path) -> Result<HashSet<String>> {
    let mut entries = match tokio::fs::read_dir(directory).await {
        Ok(entries) => entries,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
            return Ok(HashSet::new());
        }
        Err(source) => return Err(io_error(directory)(source)),
    };
    let mut identifiers = HashSet::new();
    while let Some(entry) = entries.next_entry().await.map_err(io_error(directory))? {
        let name = entry.file_name();
        if let Some(identifier) = name.to_str().and_then(|name| name.strip_suffix(".html")) {
            identifiers.insert(identifier.to_owned());
        }
        if identifiers.len() > MAX_DETAILS_PER_LANGUAGE {
            return Err(Error::CrawlLimit {
                limit: MAX_DETAILS_PER_LANGUAGE,
            });
        }
    }
    Ok(identifiers)
}

/// Returns the final path segment when it is safe to use as a file name.
pub(super) fn article_id(url: &Url) -> Option<&str> {
    url.path_segments()
        .and_then(Iterator::last)
        .filter(|segment| is_safe_identifier(segment))
}

/// Remote identifiers become file and directory names, so anything beyond the
/// observed hexadecimal form with dots is rejected instead of written to disk.
pub(super) fn is_safe_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && !value.starts_with('.')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

pub(super) fn article_links(base: &Url, document: &str, marker: &str) -> Result<HashSet<Url>> {
    Ok(html::links(base, document)?
        .into_iter()
        .filter(|url| url.host_str() == Some("www.kcna.kp") && url.path().contains(marker))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_plain_identifiers_as_file_names() -> Result<()> {
        let detail = Url::parse("http://www.kcna.kp/kp/article/detail/0011dd2005efabbd")?;
        assert_eq!(article_id(&detail), Some("0011dd2005efabbd"));
        assert!(is_safe_identifier("a514ed40.kcmsf"));
        for unsafe_value in ["", ".hidden", "..", "a/b", "a b", "a%2Fb"] {
            assert!(!is_safe_identifier(unsafe_value));
        }
        Ok(())
    }

    #[test]
    fn keeps_only_same_site_links_with_the_marker() -> Result<()> {
        let base = Url::parse("http://www.kcna.kp/kp")?;
        let document = r#"<a href="/kp/article/detail/abc">a</a>
            <a href="http://example.com/kp/article/detail/def">b</a>
            <a href="/kp/article/list/xyz">c</a>"#;
        let links = article_links(&base, document, "/article/detail/")?;
        assert_eq!(links.len(), 1);
        assert!(links.iter().all(|url| url.path().ends_with("/abc")));
        Ok(())
    }
}
