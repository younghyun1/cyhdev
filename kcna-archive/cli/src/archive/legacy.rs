//! Recursive collector for the static Korean-first kcna.co.jp archive.

use std::{
    collections::{HashSet, VecDeque},
    path::{Path, PathBuf},
    sync::Arc,
};

use futures::{StreamExt, stream::FuturesUnordered};
use serde::Deserialize;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{info, warn};
use url::Url;

use crate::{
    archive::{Summary, dead_links::DeadLinks},
    error::{Error, Result, io_error},
    fetch::Fetcher,
    html,
    manifest::Manifest,
    storage,
};

const SOURCE: &str = "kcna.co.jp";
const MAX_URLS: usize = 500_000;

/// Pages kept in flight. The fetcher's semaphore bounds actual requests; this
/// only bounds how many futures wait on it.
const WINDOW: usize = 128;

/// Pages processed between progress log lines.
const PROGRESS_INTERVAL: usize = 10_000;

#[derive(Deserialize)]
struct SeedRecord {
    url: String,
}

struct Page {
    links: Vec<Url>,
    downloaded: bool,
}

/// Where a collected page comes from and where its evidence goes.
#[derive(Clone, Copy)]
struct Sink<'a> {
    root: &'a Path,
    fetcher: &'a Fetcher,
    manifest: &'a Manifest,
    dead_links: &'a DeadLinks,
}

/// Crawls historical article and index pages, resuming from raw files already present.
pub async fn run(
    data_dir: &Path,
    manifest: Arc<Manifest>,
    fetcher: Arc<Fetcher>,
    seed_index: Option<PathBuf>,
    recheck_missing: bool,
) -> Result<Summary> {
    let root = data_dir.join("raw/kcna_co_jp");
    let dead_links = DeadLinks::open(data_dir, recheck_missing).await?;
    let sink = Sink {
        root: &root,
        fetcher: &fetcher,
        manifest: &manifest,
        dead_links: &dead_links,
    };
    let indexed = seed_index.is_some();
    let seeds = seed_urls(seed_index).await?;
    let roots = [
        Url::parse("http://www.kcna.co.jp/index-k.htm")?,
        Url::parse("http://www.kcna.co.jp/index-e.htm")?,
        Url::parse("http://www.kcna.co.jp/e-news.html")?,
    ];
    let mut queue = VecDeque::with_capacity(seeds.len().max(roots.len()));
    let mut scheduled = HashSet::with_capacity(seeds.len().max(1024));
    let initial_urls: Box<dyn Iterator<Item = Url>> = if indexed {
        Box::new(seeds.into_iter())
    } else {
        Box::new(roots.into_iter())
    };
    for url in initial_urls {
        enqueue(&mut queue, &mut scheduled, url)?;
    }
    drain_queue(sink, queue, scheduled, !indexed).await
}

/// Keeps a rolling window of pages in flight so one slow host response does
/// not stall the pages queued behind it.
async fn drain_queue(
    sink: Sink<'_>,
    mut queue: VecDeque<Url>,
    mut scheduled: HashSet<Url>,
    discover_links: bool,
) -> Result<Summary> {
    let mut summary = Summary::default();
    let mut in_flight = FuturesUnordered::new();
    let mut next_progress = PROGRESS_INTERVAL;
    loop {
        while in_flight.len() < WINDOW {
            let Some(url) = queue.pop_front() else { break };
            in_flight.push(collect_page(sink, url, discover_links));
        }
        let Some(result) = in_flight.next().await else {
            break;
        };
        match result {
            Ok(page) => {
                if page.downloaded {
                    summary.downloaded += 1;
                } else {
                    summary.reused += 1;
                }
                for url in page.links {
                    enqueue(&mut queue, &mut scheduled, url)?;
                }
            }
            Err(error) => {
                summary.record_error(&error);
                // Dead links number in the tens of thousands; only real
                // failures are worth a log line each.
                if !error.is_missing() {
                    warn!(source = SOURCE, error = %error, "Historical page failed");
                }
            }
        }
        let processed = summary.downloaded + summary.reused + summary.missing + summary.failed;
        if processed >= next_progress {
            next_progress = processed + PROGRESS_INTERVAL;
            info!(
                source = SOURCE,
                discovered = scheduled.len(),
                queued = queue.len(),
                downloaded = summary.downloaded,
                reused = summary.reused,
                missing = summary.missing,
                failed = summary.failed,
                "Historical crawl progress"
            );
        }
    }
    Ok(summary)
}

/// Schedules an archive page once. The host check runs on the original URL:
/// canonicalization rewrites the host, so checking afterwards would accept
/// every external link.
pub(super) fn enqueue(
    queue: &mut VecDeque<Url>,
    scheduled: &mut HashSet<Url>,
    url: Url,
) -> Result<()> {
    if !is_archive_html(&url) {
        return Ok(());
    }
    let canonical = canonicalize(url);
    if scheduled.insert(canonical.clone()) {
        if scheduled.len() > MAX_URLS {
            return Err(Error::CrawlLimit { limit: MAX_URLS });
        }
        queue.push_back(canonical);
    }
    Ok(())
}

async fn collect_page(sink: Sink<'_>, url: Url, discover_links: bool) -> Result<Page> {
    let path = storage::url_path(sink.root, &url);
    if !discover_links {
        // A seeded run needs no links, so a resumed page is confirmed by its
        // presence alone instead of rereading the whole corpus.
        if storage::exists(&path).await? {
            return Ok(Page {
                links: Vec::new(),
                downloaded: false,
            });
        }
        download(sink, &url, &path).await?;
        return Ok(Page {
            links: Vec::new(),
            downloaded: true,
        });
    }
    let (bytes, downloaded) = match storage::read_if_exists(&path).await? {
        Some(bytes) => (bytes, false),
        None => (download(sink, &url, &path).await?, true),
    };
    let links = html::links(&url, &html::decode(&bytes))?;
    Ok(Page { links, downloaded })
}

/// Fetches and stores one page, unless a recent run found it nonexistent.
/// A new 404 or 410 is remembered so the next run does not ask again.
async fn download(sink: Sink<'_>, url: &Url, path: &Path) -> Result<Vec<u8>> {
    if sink.dead_links.contains(url) {
        return Err(Error::KnownMissing {
            url: url.to_string(),
        });
    }
    let bytes = match sink.fetcher.get(url).await {
        Ok(bytes) => bytes,
        Err(error) => {
            if error.is_missing() {
                sink.dead_links.record(url).await?;
            }
            return Err(error);
        }
    };
    storage::write_atomic(path, &bytes).await?;
    sink.manifest.record(SOURCE, url, path, &bytes).await?;
    Ok(bytes)
}

async fn seed_urls(path: Option<PathBuf>) -> Result<VecDeque<Url>> {
    let Some(path) = path else {
        return Ok(VecDeque::new());
    };
    let file = tokio::fs::File::open(&path)
        .await
        .map_err(io_error(&path))?;
    let mut lines = BufReader::new(file).lines();
    let mut urls = VecDeque::with_capacity(360_000);
    while let Some(line) = lines.next_line().await.map_err(io_error(&path))? {
        match serde_json::from_str::<SeedRecord>(&line) {
            Ok(record) => match Url::parse(&record.url) {
                Ok(url) => urls.push_back(url),
                Err(error) => warn!(error = %error, "Skipping malformed seed URL"),
            },
            Err(error) => warn!(error = %error, "Skipping malformed seed record"),
        }
    }
    info!(seed = %path.display(), urls = urls.len(), "Loaded public archive index");
    Ok(urls)
}

/// Collapses scheme, bare-host, query, and fragment variants of one page.
/// Callers must have accepted the URL with [`is_archive_html`] first.
fn canonicalize(mut url: Url) -> Url {
    let _ = url.set_scheme("http");
    let _ = url.set_host(Some("www.kcna.co.jp"));
    let _ = url.set_port(None);
    url.set_fragment(None);
    url.set_query(None);
    url
}

pub(super) fn is_archive_html(url: &Url) -> bool {
    let file_name = url.path().rsplit('/').next().unwrap_or_default();
    matches!(url.scheme(), "http" | "https")
        && matches!(url.host_str(), Some("kcna.co.jp" | "www.kcna.co.jp"))
        && file_name.rsplit_once('.').is_some_and(|(stem, extension)| {
            // The mirror's indexes contain a link written as `.htm#16`. A file
            // name without a stem is that typo, not a page, and the host
            // answers it with 403.
            !stem.is_empty()
                && (extension.eq_ignore_ascii_case("htm") || extension.eq_ignore_ascii_case("html"))
        })
}
