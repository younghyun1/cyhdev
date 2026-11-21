//! Blocking load and parse of one historical page.

use std::path::{Path, PathBuf};

use tracing::warn;
use url::Url;

use crate::{
    error::{Result, io_error},
    html,
    importer::{
        common::sha256,
        extract::{self, ArticleText},
        provenance::{Provenance, ProvenanceIndex},
    },
    storage,
};

/// One raw page, parsed once for every record that points at it.
pub struct Page {
    pub url: Url,
    pub bytes: Vec<u8>,
    pub charset: String,
    pub raw_sha256: [u8; 32],
    pub articles: Vec<ArticleText>,
    pub retrieval: Provenance,
}

impl Page {
    /// Number of article slots found on the page.
    pub const fn article_count(&self) -> usize {
        self.articles.len()
    }
}

pub fn load_page(root: &Path, provenance: &ProvenanceIndex, source_url: &str) -> Result<Page> {
    let url = canonical_url(source_url)?;
    let path = storage::url_path(root, &url);
    let bytes = std::fs::read(&path).map_err(io_error(&path))?;
    let mut articles = extract::legacy(&html::decode(&bytes))?;
    resolve_index_titles(&path, &mut articles);
    let raw_sha256 = sha256(&bytes);
    let retrieval = provenance.resolve(&path, raw_sha256, url.as_str())?;
    Ok(Page {
        charset: html::charset(&bytes),
        url,
        bytes,
        raw_sha256,
        articles,
        retrieval,
    })
}

/// Fills titles that a multi-article page omits from its sibling daily index.
fn resolve_index_titles(path: &Path, articles: &mut Vec<ArticleText>) {
    if !path
        .components()
        .any(|component| component.as_os_str() == "item")
    {
        return;
    }
    let Some(index_path) = daily_index_path(path) else {
        return;
    };
    let bytes = match std::fs::read(&index_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            warn!(path = %index_path.display(), error = %error, "Historical daily index is unavailable");
            return;
        }
    };
    let titles = match extract::legacy_index_titles(&html::decode(&bytes)) {
        Ok(titles) => titles,
        Err(error) => {
            warn!(path = %index_path.display(), error = %error, "Historical daily index is invalid");
            return;
        }
    };
    if let Some(last_fragment) = titles.keys().max().copied()
        && articles.len() <= last_fragment
    {
        articles.resize_with(last_fragment.saturating_add(1), ArticleText::default);
    }
    for (index, article) in articles.iter_mut().enumerate() {
        if !article.title.is_empty() {
            continue;
        }
        let Some(title) = titles.get(&index) else {
            continue;
        };
        if let Some(body) = article.body.strip_prefix(title.as_str()) {
            article.body = body.trim_start().to_owned();
        }
        article.title.clone_from(title);
    }
}

fn daily_index_path(path: &Path) -> Option<PathBuf> {
    let day = path.file_stem()?.to_str()?;
    if day.len() > 2 || !day.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let month_directory = path.parent()?.parent()?;
    let month = month_directory.file_name()?.to_str()?;
    Some(month_directory.join(format!("{month}{day}.htm")))
}

fn canonical_url(value: &str) -> Result<Url> {
    let mut url = Url::parse(value)?;
    let _ = url.set_scheme("http");
    let _ = url.set_host(Some("www.kcna.co.jp"));
    url.set_fragment(None);
    url.set_query(None);
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locates_the_daily_index_beside_a_news_directory() {
        let page = Path::new("raw/kcna_co_jp/item/2002/200211/news11/15.htm");
        assert_eq!(
            daily_index_path(page),
            Some(PathBuf::from(
                "raw/kcna_co_jp/item/2002/200211/20021115.htm"
            ))
        );
        assert_eq!(daily_index_path(Path::new("a/b/2002-11-18-001.htm")), None);
    }
}
