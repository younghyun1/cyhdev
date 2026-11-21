//! Category pagination for the live collector: enumeration and date evidence.

use std::{collections::HashSet, path::Path, sync::LazyLock};

use futures::{StreamExt, stream};
use jiff::Timestamp;
use regex::Regex;
use tracing::warn;
use url::Url;

use crate::{
    archive::{
        Summary,
        live::{Edition, SOURCE, article_id, article_links, is_safe_identifier},
    },
    error::{Error, Result},
    html, storage,
};

/// Upper bound on one category's pagination; far above the few hundred pages
/// observed, and low enough to stop a misparsed counter from looping.
const MAX_LIST_PAGES: usize = 20_000;
const LIST_WINDOW: usize = 128;

static PAGE_COUNT: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"var\s+page_cnt\s*=\s*(\d+)").ok());

struct ListPage {
    links: HashSet<Url>,
    stored: bool,
}

/// Fetches every list page of a category from the site on each run.
///
/// Pagination shifts whenever KCNA publishes, so a page number cached from an
/// earlier run no longer describes the same articles. Reusing cached pages
/// would silently skip everything published between runs beyond page one.
pub(super) async fn enumerate_category(
    context: &Edition<'_>,
    category: &Url,
) -> Result<(HashSet<Url>, Summary)> {
    let first = context.fetcher.get(category).await?;
    let document = html::decode(&first);
    let csrf = html::input_value(&document, "_csrf")?.ok_or(Error::MissingField("_csrf"))?;
    let page_count = page_count(&document)?;
    if page_count > MAX_LIST_PAGES {
        return Err(Error::CrawlLimit {
            limit: MAX_LIST_PAGES,
        });
    }
    let category_id = category
        .path_segments()
        .and_then(Iterator::last)
        .filter(|segment| is_safe_identifier(segment))
        .ok_or(Error::MissingField("category id"))?;
    let directory = context.data_dir.join(format!(
        "raw/kcna_kp/lists/{}/{category_id}",
        context.edition.path()
    ));
    // One timestamp per enumeration keeps a run's snapshots grouped.
    let run_stamp = Timestamp::now().as_second();
    let mut details = article_links(category, &document, "/article/detail/")?;
    let mut summary = Summary::default();
    let stored = store_list_page(
        context, category, &directory, 1, run_stamp, &first, &details,
    )
    .await?;
    count_list_page(&mut summary, stored);
    let results = stream::iter(2..=page_count)
        .map(|page| {
            let csrf = csrf.clone();
            let directory = directory.as_path();
            async move {
                let form = [
                    ("page_num", page.to_string()),
                    ("cnt_per_page", "15".to_owned()),
                    ("keyword", String::new()),
                    ("_csrf", csrf),
                ];
                let bytes = context.fetcher.post_form(category, &form).await?;
                let links = article_links(category, &html::decode(&bytes), "/article/detail/")?;
                let stored = store_list_page(
                    context, category, directory, page, run_stamp, &bytes, &links,
                )
                .await?;
                Ok::<_, Error>(ListPage { links, stored })
            }
        })
        .buffer_unordered(LIST_WINDOW)
        .collect::<Vec<_>>()
        .await;
    for result in results {
        match result {
            Ok(page) => {
                details.extend(page.links);
                count_list_page(&mut summary, page.stored);
            }
            Err(error) => {
                summary.failed += 1;
                warn!(
                    language = context.edition.path(),
                    category = %category,
                    error = %error,
                    "Live list page failed"
                );
            }
        }
    }
    Ok((details, summary))
}

fn count_list_page(summary: &mut Summary, stored: bool) {
    if stored {
        summary.downloaded += 1;
    } else {
        summary.reused += 1;
    }
}

/// Keeps a list page when it is evidence the corpus does not hold yet.
///
/// List pages are the only source of publication dates, so the importer needs
/// one stored page naming every article. The first snapshot of a page number
/// keeps the plain `{page}.html` name. Later runs add a timestamped snapshot
/// only when the page names an article that is not stored, and never replace
/// an earlier one: pagination shifts, so overwriting would drop the dates of
/// the articles that moved to the next page.
async fn store_list_page(
    context: &Edition<'_>,
    category: &Url,
    directory: &Path,
    page: usize,
    run_stamp: i64,
    bytes: &[u8],
    links: &HashSet<Url>,
) -> Result<bool> {
    let numbered = directory.join(format!("{page}.html"));
    let path = if !storage::exists(&numbered).await? {
        numbered
    } else if links.iter().any(|url| !is_known(context, url)) {
        directory.join(format!("{page}.r{run_stamp}.html"))
    } else {
        return Ok(false);
    };
    storage::write_atomic(&path, bytes).await?;
    context
        .manifest
        .record(SOURCE, category, &path, bytes)
        .await?;
    Ok(true)
}

fn is_known(context: &Edition<'_>, url: &Url) -> bool {
    article_id(url).is_some_and(|id| context.known.contains(id))
}

fn page_count(document: &str) -> Result<usize> {
    let expression = PAGE_COUNT
        .as_ref()
        .ok_or_else(|| Error::Selector("page count expression is invalid".to_owned()))?;
    let captures = expression
        .captures(document)
        .ok_or(Error::MissingField("page_cnt"))?;
    captures
        .get(1)
        .and_then(|value| value.as_str().parse().ok())
        .ok_or(Error::MissingField("numeric page_cnt"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_page_count_with_legacy_spacing() -> Result<()> {
        assert_eq!(page_count("var page_cnt\t=\t277;")?, 277);
        assert!(page_count("var total = 3;").is_err());
        Ok(())
    }
}
