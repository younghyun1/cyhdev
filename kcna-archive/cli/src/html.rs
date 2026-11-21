//! HTML decoding and link extraction for KCNA's mixed legacy encodings.

use std::{borrow::Cow, sync::LazyLock};

use encoding_rs::{EUC_KR, Encoding, UTF_8, UTF_16BE, UTF_16LE, WINDOWS_1252};
use regex::bytes::Regex;
use scraper::{Html, Selector};
use url::Url;

use crate::error::{Error, Result};

/// Declarations sit in the document head; scanning further only finds body text.
const DECLARATION_PROBE_BYTES: usize = 2048;

/// Undeclared pages with at least this share of non-ASCII bytes are Korean
/// text; English and Spanish pages carry only stray Latin-1 punctuation.
const CJK_HIGH_BYTE_PERCENT: usize = 5;

/// Tolerated share of undecodable sequences in an otherwise Korean page.
const EUC_KR_ERROR_PERCENT: usize = 5;

static CHARSET_LABEL: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r#"(?i)charset\s*=\s*["']?([a-z0-9_:.-]+)"#).ok());

/// Decodes modern UTF-8 and historical EUC-KR, Shift-JIS, Latin-1, and UTF-16 pages.
pub fn decode(bytes: &[u8]) -> Cow<'_, str> {
    // `Encoding::decode` strips a matching byte-order mark itself.
    encoding(bytes).decode(bytes).0
}

/// Reports the source charset used to decode a raw page.
pub fn charset(bytes: &[u8]) -> String {
    encoding(bytes).name().to_ascii_lowercase()
}

/// Selects the decoder: byte-order mark, then the declared label, then content.
///
/// A declared UTF-8 label is trusted only when the bytes validate, and an
/// absent label falls back to content sniffing, because roughly 1,800
/// historical Korean pages are EUC-KR without any declaration.
fn encoding(bytes: &[u8]) -> &'static Encoding {
    if let Some((encoding, _)) = Encoding::for_bom(bytes) {
        return encoding;
    }
    match declared(bytes) {
        Some(declared) if declared != UTF_8 => declared,
        Some(_) | None => sniff(bytes),
    }
}

fn declared(bytes: &[u8]) -> Option<&'static Encoding> {
    let probe = &bytes[..bytes.len().min(DECLARATION_PROBE_BYTES)];
    let label = CHARSET_LABEL
        .as_ref()?
        .captures(probe)
        .and_then(|captures| captures.get(1))?;
    // WHATWG labels cover the historical aliases `ks_c_5601-1987` and `x-sjis`
    // and map `iso-8859-1` to its windows-1252 superset.
    let encoding = Encoding::for_label(label.as_bytes())?;
    // UTF-16 without a byte-order mark cannot carry an ASCII declaration, so a
    // label naming it is wrong by construction.
    (encoding != UTF_16LE && encoding != UTF_16BE).then_some(encoding)
}

fn sniff(bytes: &[u8]) -> &'static Encoding {
    if std::str::from_utf8(bytes).is_ok() {
        return UTF_8;
    }
    let high_bytes = bytes.iter().filter(|byte| **byte >= 0x80).count();
    if high_bytes.saturating_mul(100) < bytes.len().saturating_mul(CJK_HIGH_BYTE_PERCENT) {
        return WINDOWS_1252;
    }
    let (decoded, _, had_errors) = EUC_KR.decode(bytes);
    if !had_errors {
        return EUC_KR;
    }
    let replacements = decoded.matches('\u{fffd}').count();
    // Each Korean character spans two high bytes.
    if replacements.saturating_mul(200) <= high_bytes.saturating_mul(EUC_KR_ERROR_PERCENT) {
        EUC_KR
    } else {
        WINDOWS_1252
    }
}

/// Resolves anchor and frame links against the page URL.
pub fn links(base: &Url, document: &str) -> Result<Vec<Url>> {
    let selector = Selector::parse("a[href], frame[src]")
        .map_err(|error| Error::Selector(error.to_string()))?;
    let html = Html::parse_document(document);
    Ok(html
        .select(&selector)
        .filter_map(|node| {
            node.value()
                .attr("href")
                .or_else(|| node.value().attr("src"))
        })
        .filter_map(|link| base.join(link).ok())
        .collect())
}

/// Extracts an input value by its name.
pub fn input_value(document: &str, name: &str) -> Result<Option<String>> {
    let selector = Selector::parse("input").map_err(|error| Error::Selector(error.to_string()))?;
    let html = Html::parse_document(document);
    Ok(html.select(&selector).find_map(|node| {
        (node.value().attr("name") == Some(name))
            .then(|| node.value().attr("value").unwrap_or_default().to_owned())
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    const KOREAN: &str = "조선중앙통신 평양발 보도문을 전한다";

    fn euc_kr(text: &str) -> Vec<u8> {
        EUC_KR.encode(text).0.into_owned()
    }

    #[test]
    fn decodes_declared_euc_kr_aliases() {
        for label in ["euc-kr", "EUC-KR", "ks_c_5601-1987"] {
            let mut page =
                format!("<meta http-equiv=Content-Type content=\"text/html; charset={label}\">")
                    .into_bytes();
            page.extend(euc_kr(KOREAN));
            assert_eq!(charset(&page), "euc-kr");
            assert!(decode(&page).ends_with(KOREAN));
        }
    }

    #[test]
    fn decodes_undeclared_korean_pages_as_euc_kr() {
        let mut page = b"<html><body><h4>".to_vec();
        page.extend(euc_kr(KOREAN));
        page.extend(b"</h4></body></html>");
        assert_eq!(charset(&page), "euc-kr");
        assert!(decode(&page).contains(KOREAN));
    }

    #[test]
    fn keeps_valid_utf8_without_declaration() {
        let page = format!("<html><body>{KOREAN}</body></html>");
        assert_eq!(charset(page.as_bytes()), "utf-8");
        assert_eq!(decode(page.as_bytes()), page);
    }

    #[test]
    fn decodes_stray_latin1_in_mostly_ascii_pages() {
        let mut page = b"<html><body>".to_vec();
        page.extend(b"Pyongyang, January 29 (KCNA) -- The Jos".repeat(4));
        page.push(0xe9);
        page.extend(b" delegation arrived here today.</body></html>");
        assert_eq!(charset(&page), "windows-1252");
        assert!(decode(&page).contains("Jos\u{e9} delegation"));
    }

    #[test]
    fn maps_declared_latin1_and_shift_jis_labels() {
        let latin = b"<meta charset=iso-8859-1><p>Espa\xf1a</p>";
        assert_eq!(charset(latin), "windows-1252");
        assert!(decode(latin).contains("Espa\u{f1}a"));
        let japanese = b"<meta charset=\"x-sjis\"><p>\x92\xa9\x91N</p>";
        assert_eq!(charset(japanese), "shift_jis");
        assert!(decode(japanese).contains("朝鮮"));
    }

    #[test]
    fn honours_utf16_byte_order_marks() {
        let mut page = vec![0xff, 0xfe];
        for unit in "평양".encode_utf16() {
            page.extend(unit.to_le_bytes());
        }
        assert_eq!(charset(&page), "utf-16le");
        assert_eq!(decode(&page), "평양");
    }

    #[test]
    fn resolves_relative_links_and_frames() -> Result<()> {
        let base = Url::parse("http://www.kcna.co.jp/calendar/2002/index.htm")?;
        let anchors = links(&base, "<a href=\"a.htm\">a</a><a name=\"1\">b</a>")?;
        assert_eq!(
            anchors.iter().map(Url::as_str).collect::<Vec<_>>(),
            ["http://www.kcna.co.jp/calendar/2002/a.htm"]
        );
        // The parser keeps `frame` elements only inside a frameset document.
        let frames = links(
            &base,
            "<html><frameset><frame src=\"/b.htm\"></frameset></html>",
        )?;
        assert_eq!(
            frames.iter().map(Url::as_str).collect::<Vec<_>>(),
            ["http://www.kcna.co.jp/b.htm"]
        );
        Ok(())
    }
}
