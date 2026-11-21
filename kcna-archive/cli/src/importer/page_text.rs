//! Text recovery from the table and line-break layouts of historical pages.

use std::sync::LazyLock;

use regex::Regex;
use scraper::{ElementRef, Html, Node};

use crate::error::{Error, Result};

// The footer every Korea News Service page repeats after the article.
static COPYRIGHT_NOTICE: LazyLock<Option<Regex>> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\s*Copyright\s*\(C\)\s*(?:\d{4}(?:\s*-\s*\d{4})?)?\s*KOREA NEWS SERVICE\s*\(KNS\)\s*All Rights Reserved\.?\s*$",
    )
    .ok()
});

/// Characters of page text searched for the title. Only the breadcrumb sits
/// before it; a later occurrence is the article quoting its own headline.
const TITLE_SEARCH_CHARS: usize = 400;

/// Resolves a built-in expression, reporting a compile failure as an error.
pub(super) fn expression(cell: &'static LazyLock<Option<Regex>>) -> Result<&'static Regex> {
    cell.as_ref()
        .ok_or_else(|| Error::Selector("built-in expression is invalid".to_owned()))
}

/// Joins paragraphs into a body, without the leading breadcrumb and title and
/// the trailing copyright notice.
pub(super) fn trim_page_chrome(paragraphs: Vec<String>, title: &str) -> Result<String> {
    // The title is matched against the paragraphs joined by spaces because a
    // headline broken over two lines arrives as two paragraphs.
    let flat = paragraphs.join(" ");
    let window = flat
        .char_indices()
        .nth(TITLE_SEARCH_CHARS)
        .map_or(flat.len(), |(index, _)| index);
    let skip = match (title.is_empty(), flat[..window].find(title)) {
        (false, Some(index)) => index + title.len(),
        _ => 0,
    };
    let mut kept = Vec::with_capacity(paragraphs.len());
    let mut offset = 0_usize;
    for paragraph in &paragraphs {
        let end = offset + paragraph.len();
        if offset >= skip {
            kept.push(paragraph.as_str());
        } else if end > skip {
            // `skip` is a character boundary of `flat`, and this paragraph is
            // the same bytes at `offset`, so the slice cannot split a character.
            let rest = paragraph[skip - offset..].trim_start();
            if !rest.is_empty() {
                kept.push(rest);
            }
        }
        offset = end + 1;
    }
    let body = kept.join("\n\n");
    let body = match expression(&COPYRIGHT_NOTICE)?.find(&body) {
        Some(notice) => &body[..notice.start()],
        None => body.as_str(),
    };
    Ok(body.trim().to_owned())
}

/// Elements that end a paragraph in the table and line-break layouts of the
/// historical pages.
const BLOCK_ELEMENTS: [&str; 19] = [
    "p",
    "br",
    "div",
    "table",
    "tr",
    "td",
    "li",
    "ul",
    "ol",
    "hr",
    "center",
    "blockquote",
    "pre",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
];

enum Step<'a> {
    Enter(ElementRef<'a>),
    Text(&'a str),
    Leave(&'a str),
}

/// Collects an element's text as whitespace-normalized paragraphs.
///
/// Iterative, because malformed pages nest arbitrarily deep.
pub(super) fn paragraphs(root: ElementRef<'_>) -> Vec<String> {
    let mut paragraphs = Vec::new();
    let mut current = String::new();
    let mut flush = |current: &mut String| {
        let paragraph = normalize(std::iter::once(current.as_str()));
        if !paragraph.is_empty() {
            paragraphs.push(paragraph);
        }
        current.clear();
    };
    let mut stack = vec![Step::Enter(root)];
    while let Some(step) = stack.pop() {
        match step {
            Step::Text(text) => current.push_str(text),
            Step::Leave(name) => {
                if BLOCK_ELEMENTS.contains(&name) {
                    flush(&mut current);
                }
            }
            Step::Enter(element) => {
                let name = element.value().name();
                if matches!(name, "script" | "style") {
                    continue;
                }
                if BLOCK_ELEMENTS.contains(&name) {
                    flush(&mut current);
                }
                stack.push(Step::Leave(name));
                // Reversed so children pop in document order.
                for child in element.children().rev() {
                    match child.value() {
                        Node::Text(text) => stack.push(Step::Text(text)),
                        Node::Element(_) => {
                            if let Some(child) = ElementRef::wrap(child) {
                                stack.push(Step::Enter(child));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    flush(&mut current);
    paragraphs
}

pub(super) fn fragment_text(fragment: &str) -> String {
    normalize(Html::parse_fragment(fragment).root_element().text())
}

pub(super) fn normalize<'a>(parts: impl Iterator<Item = &'a str>) -> String {
    parts
        .flat_map(str::split_whitespace)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leaves_a_title_quoted_deep_in_the_body_alone() -> Result<()> {
        let filler = "가".repeat(TITLE_SEARCH_CHARS + 10);
        let text = format!("{filler} 제목 뒤의 글");
        assert_eq!(trim_page_chrome(vec![text.clone()], "제목")?, text);
        assert_eq!(
            trim_page_chrome(vec!["목록 제목 본문".to_owned()], "제목")?,
            "본문"
        );
        let split_title = vec![
            "목록".to_owned(),
            "긴".to_owned(),
            "제목".to_owned(),
            "첫 문단".to_owned(),
            "둘째 문단".to_owned(),
        ];
        assert_eq!(
            trim_page_chrome(split_title, "긴 제목")?,
            "첫 문단\n\n둘째 문단"
        );
        Ok(())
    }
}
