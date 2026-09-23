//! `If-None-Match` parsing for bundle revalidation.

use axum::http::{HeaderMap, header};

use super::super::domain::entity_tag::{IfNoneMatch, MAX_IF_NONE_MATCH_TAGS};

/// Parses every `If-None-Match` header into opaque tags.
///
/// Weak prefixes are dropped because the header uses weak comparison.
/// Unquoted or malformed members are ignored rather than rejected, which at
/// worst sends the full bundle again. At most [`MAX_IF_NONE_MATCH_TAGS`] tags
/// are kept.
pub(super) fn parse_if_none_match(headers: &HeaderMap) -> IfNoneMatch {
    let mut tags = Vec::new();
    let mut present = false;
    for value in headers.get_all(header::IF_NONE_MATCH) {
        present = true;
        let Ok(value) = value.to_str() else { continue };
        for member in value.split(',').map(str::trim) {
            if member == "*" {
                return IfNoneMatch::Any;
            }
            let member = member.strip_prefix("W/").unwrap_or(member);
            if let Some(tag) = member
                .strip_prefix('"')
                .and_then(|rest| rest.strip_suffix('"'))
                && tags.len() < MAX_IF_NONE_MATCH_TAGS
            {
                tags.push(tag.to_owned());
            }
        }
    }
    if present {
        IfNoneMatch::Tags(tags)
    } else {
        IfNoneMatch::Absent
    }
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderMap, HeaderValue, header};

    use super::{IfNoneMatch, parse_if_none_match};

    fn headers(values: &[&'static str]) -> HeaderMap {
        let mut headers = HeaderMap::new();
        for value in values {
            headers.append(header::IF_NONE_MATCH, HeaderValue::from_static(value));
        }
        headers
    }

    #[test]
    fn parses_lists_weak_tags_and_wildcards() {
        assert_eq!(parse_if_none_match(&HeaderMap::new()), IfNoneMatch::Absent);
        assert_eq!(parse_if_none_match(&headers(&["*"])), IfNoneMatch::Any);
        assert_eq!(
            parse_if_none_match(&headers(&[r#""a", W/"b""#, r#"unquoted, "c""#])),
            IfNoneMatch::Tags(vec!["a".to_owned(), "b".to_owned(), "c".to_owned()])
        );
    }

    #[test]
    fn keeps_a_bounded_number_of_tags() {
        let many = (0..100)
            .map(|index| format!(r#""t{index}""#))
            .collect::<Vec<_>>()
            .join(", ");
        let mut map = HeaderMap::new();
        map.insert(
            header::IF_NONE_MATCH,
            HeaderValue::from_str(&many).unwrap_or(HeaderValue::from_static("")),
        );
        assert!(matches!(
            parse_if_none_match(&map),
            IfNoneMatch::Tags(tags) if tags.len() == 32
        ));
    }
}
