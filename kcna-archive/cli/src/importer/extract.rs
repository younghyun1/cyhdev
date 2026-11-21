//! Text extraction for semantic live pages and irregular historical HTML.

use std::{collections::HashMap, sync::LazyLock};

use regex::Regex;
use scraper::{ElementRef, Html, Selector};

use crate::{
    error::{Error, Result},
    importer::page_text::{expression, fragment_text, normalize, paragraphs, trim_page_chrome},
};

/// Highest article anchor accepted on one page. Real pages number articles up
/// to a few dozen; the bound stops a stray anchor such as `name="20020101"`
/// from allocating millions of empty slots.
pub const MAX_PAGE_ARTICLES: usize = 1_024;

// Old KCNA pages disagree about whether the anchor or heading closes first;
// some start the body on the title line, omit both closing tags, or wrap the
// anchor inside a multiline heading.
static ARTICLE_MARKER: LazyLock<Option<Regex>> = LazyLock::new(|| {
    Regex::new(
        r#"(?:(?is:<a\s+name\s*=\s*[\"']?\d+[\"']?[^>]*>\s*<h[1-6][^>]*>([^\r\n]*?)(?:</h[1-6]>\s*</a>|</a>\s*</h[1-6]>|</h[1-6]>|</a>))|(?im:<a\s+name\s*=\s*[\"']?\d+[\"']?[^>]*>\s*<h[1-6][^>]*>([^\r\n]*)\r?$)|(?is:<h[1-6][^>]*>\s*<a\s+name\s*=\s*[\"']?\d+[\"']?[^>]*>(.*?)</a>\s*</h[1-6]>)|(?is:<a\s+name\s*=\s*[\"']?\d+[\"']?[^>]*>([^<\r\n]+)<h[1-6][^>]*>\s*</h[1-6]>\s*</a>))"#,
    )
    .ok()
});
static ANCHOR_NUMBER: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r#"(?i)name\s*=\s*[\"']?(\d+)"#).ok());

/// Parses a selector that is a compile-time constant in this module.
fn selector(source: &'static str) -> Result<Selector> {
    Selector::parse(source).map_err(|error| Error::Selector(error.to_string()))
}

#[derive(Clone, Debug, Default)]
pub struct ArticleText {
    pub title: String,
    pub body: String,
}

/// Extracts one article from the current semantic KCNA markup.
pub fn live(document: &str) -> Result<ArticleText> {
    let html = Html::parse_document(document);
    let heading = selector("article h1")?;
    let paragraph = selector("article p")?;
    let title = html
        .select(&heading)
        .map(|node| normalize(node.text()))
        .find(|value| !value.is_empty())
        .ok_or_else(|| Error::InvalidRecord("live article title is missing".to_owned()))?;
    let body = html
        .select(&paragraph)
        .map(|node| normalize(node.text()))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");
    if body.is_empty() {
        return Err(Error::InvalidRecord(
            "live article body is missing".to_owned(),
        ));
    }
    Ok(ArticleText { title, body })
}

/// Splits old multi-article pages and falls back to the single Korean article layout.
pub fn legacy(document: &str) -> Result<Vec<ArticleText>> {
    let marker = expression(&ARTICLE_MARKER)?;
    let anchor_number = expression(&ANCHOR_NUMBER)?;
    let matches = marker.captures_iter(document).collect::<Vec<_>>();
    if !matches.is_empty() {
        let mut articles = Vec::with_capacity(matches.len());
        for (index, captures) in matches.iter().enumerate() {
            let full = captures.get(0).ok_or_else(|| {
                Error::InvalidRecord("legacy article marker is missing".to_owned())
            })?;
            let title_fragment = captures
                .get(1)
                .or_else(|| captures.get(2))
                .or_else(|| captures.get(3))
                .or_else(|| captures.get(4))
                .ok_or_else(|| {
                    Error::InvalidRecord("legacy article title is missing".to_owned())
                })?;
            let body_end = matches
                .get(index + 1)
                .and_then(|next| next.get(0))
                .map_or(document.len(), |next| next.start());
            let title = fragment_text(title_fragment.as_str());
            let fragment = Html::parse_fragment(&document[full.end()..body_end]);
            let body = trim_page_chrome(paragraphs(fragment.root_element()), "")?;
            // An anchor outside the bound is not an article number; skip it
            // instead of failing the page's other articles.
            let Some(fragment) = anchor_number
                .captures(full.as_str())
                .and_then(|captures| captures.get(1))
                .and_then(|value| value.as_str().parse::<usize>().ok())
                .and_then(|value| value.checked_sub(1))
                .filter(|fragment| *fragment < MAX_PAGE_ARTICLES)
            else {
                continue;
            };
            if articles.len() <= fragment {
                articles.resize_with(fragment.saturating_add(1), ArticleText::default);
            }
            let candidate = ArticleText { title, body };
            if article_score(&candidate) > article_score(&articles[fragment]) {
                articles[fragment] = candidate;
            }
        }
        if !articles.is_empty() {
            return Ok(articles);
        }
    }
    single_legacy(document).map(|article| vec![article])
}

/// Extracts fragment-numbered titles from a historical daily index page.
pub fn legacy_index_titles(document: &str) -> Result<HashMap<usize, String>> {
    let html = Html::parse_document(document);
    let headings = selector("h1, h2, h3, h4, h5, h6")?;
    let links = selector("a[href]")?;
    let mut titles = HashMap::new();
    for heading in html.select(&headings) {
        let Some(link) = heading.select(&links).next() else {
            continue;
        };
        let Some(fragment) = link
            .value()
            .attr("href")
            .and_then(|href| href.rsplit_once('#'))
            .map(|(_, fragment)| fragment)
            .and_then(|fragment| fragment.parse::<usize>().ok())
            .and_then(|fragment| fragment.checked_sub(1))
            .filter(|fragment| *fragment < MAX_PAGE_ARTICLES)
        else {
            continue;
        };
        let title = normalize(heading.text());
        if !title.is_empty() {
            titles.insert(fragment, title);
        }
    }
    Ok(titles)
}

fn article_score(article: &ArticleText) -> (bool, bool, usize) {
    (
        !article.title.is_empty(),
        !article.body.is_empty(),
        article.title.len().saturating_add(article.body.len()),
    )
}

/// True for bold text that belongs to the breadcrumb: it links somewhere, sits
/// inside a link, or carries the `>>` separator.
fn is_navigation(node: ElementRef<'_>, link: &Selector) -> bool {
    node.select(link).next().is_some()
        || node
            .ancestors()
            .filter_map(ElementRef::wrap)
            .any(|ancestor| ancestor.value().name() == "a")
        || node.text().any(|text| text.contains(">>"))
}

fn single_legacy(document: &str) -> Result<ArticleText> {
    let html = Html::parse_document(document);
    let bold = selector("b")?;
    let aligned = selector("div[align=\"left\"]")?;
    let aligned_bold = selector("div[align=\"left\"] b")?;
    let link = selector("a")?;
    let content_cell = selector("td[width=\"650\"]")?;
    let body_selector = selector("body")?;
    // Pages open with a bold navigation breadcrumb (`calendar >> June 19.
    // 2025`), so the first bold text is not the headline. The headline is the
    // bold text of the left-aligned title block; older layouts without that
    // block fall back to the first bold text that is not navigation.
    let first_text = |selector: &Selector, skip_navigation: bool| {
        html.select(selector)
            .filter(|node| !(skip_navigation && is_navigation(*node, &link)))
            .map(|node| normalize(node.text()))
            .find(|value| !value.is_empty())
    };
    let title = first_text(&aligned_bold, false)
        .or_else(|| first_text(&bold, true))
        .or_else(|| first_text(&aligned, false))
        .ok_or_else(|| Error::InvalidRecord("legacy article title is missing".to_owned()))?;
    let page = html
        .select(&content_cell)
        .map(paragraphs)
        .filter(|cell| !cell.is_empty() && cell.join(" ") != title)
        .max_by_key(|cell| cell.iter().map(String::len).sum::<usize>())
        .or_else(|| html.select(&body_selector).next().map(paragraphs))
        .unwrap_or_default();
    // Layouts without a recognizable content cell yield the whole page, so the
    // navigation breadcrumb, the repeated title, and the footer are removed.
    // A page that is only a headline keeps an empty body for the caller to mark.
    let body = trim_page_chrome(page, &title)?;
    Ok(ArticleText { title, body })
}
