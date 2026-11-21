//! Marks query matches in displayed text and cuts result snippets.

use std::ops::Range;

use crate::search_text::{self, Clause};

/// Characters of text shown in a result row.
pub const SNIPPET_CHARS: usize = 170;
/// Characters of context kept before the first match in a snippet.
const LEAD_CHARS: usize = 36;

/// A run of text that is entirely inside or outside a match.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Segment {
    pub text: String,
    pub hit: bool,
}

/// Byte ranges of `text` matched by any clause, sorted and non-overlapping.
///
/// Uses the same lexemes as the index, so what is marked is exactly what
/// matched: a phrase marks its whole span and a one-character query marks
/// that character wherever a lexeme starts with it.
pub fn matches(text: &str, clauses: &[Clause]) -> Vec<Range<usize>> {
    if clauses.is_empty() || text.is_empty() {
        return Vec::new();
    }
    let mut lexemes = Vec::new();
    search_text::document_spans(text, |lexeme, range| {
        lexemes.push((lexeme.to_owned(), range))
    });
    let mut ranges = Vec::new();
    for clause in clauses {
        match clause {
            Clause::Term(term) => ranges.extend(
                lexemes
                    .iter()
                    .filter(|(lexeme, _)| lexeme == term)
                    .map(|(_, range)| range.clone()),
            ),
            Clause::Prefix(prefix) => ranges.extend(
                lexemes
                    .iter()
                    .filter(|(lexeme, _)| lexeme.starts_with(prefix.as_str()))
                    // Only the queried character matched, not the whole bigram.
                    .map(|(_, range)| range.start..range.start + prefix.len()),
            ),
            Clause::Phrase(phrase) => {
                for window in lexemes.windows(phrase.len().max(1)) {
                    let equal = window
                        .iter()
                        .zip(phrase)
                        .all(|((lexeme, _), expected)| lexeme == expected);
                    if let (true, Some(first), Some(last)) = (equal, window.first(), window.last())
                    {
                        ranges.push(first.1.start..last.1.end);
                    }
                }
            }
        }
    }
    merge(ranges)
}

fn merge(mut ranges: Vec<Range<usize>>) -> Vec<Range<usize>> {
    ranges.sort_unstable_by_key(|range| (range.start, range.end));
    let mut merged: Vec<Range<usize>> = Vec::with_capacity(ranges.len());
    for range in ranges {
        match merged.last_mut() {
            Some(last) if range.start <= last.end => last.end = last.end.max(range.end),
            _ => merged.push(range),
        }
    }
    merged
}

/// Splits `text` into alternating plain and matched segments.
pub fn segments(text: &str, clauses: &[Clause]) -> Vec<Segment> {
    cut(text, 0..text.len(), &matches(text, clauses))
}

/// Cuts a window of about [`SNIPPET_CHARS`] around the first match, or from
/// the start when nothing in `text` matched, with ellipses where it was cut.
pub fn snippet(text: &str, clauses: &[Clause]) -> Vec<Segment> {
    let ranges = matches(text, clauses);
    let first = ranges.first().map_or(0, |range| range.start);
    let start = back_by_chars(text, first, LEAD_CHARS);
    // Start on a word where possible so the snippet does not open mid-word.
    let start = match text[start..first].find(char::is_whitespace) {
        Some(space) if start > 0 => start + space + 1,
        _ => start,
    };
    let end = forward_by_chars(text, start, SNIPPET_CHARS);
    let mut segments = cut(text, start..end, &ranges);
    if start > 0 {
        segments.insert(
            0,
            Segment {
                text: "…".to_owned(),
                hit: false,
            },
        );
    }
    if end < text.len() {
        segments.push(Segment {
            text: "…".to_owned(),
            hit: false,
        });
    }
    segments
}

/// Segments of `text[window]`, with matches clipped to the window and every
/// run of whitespace (including paragraph breaks) collapsed to one space.
fn cut(text: &str, window: Range<usize>, ranges: &[Range<usize>]) -> Vec<Segment> {
    let mut pieces: Vec<(Range<usize>, bool)> = Vec::new();
    let mut cursor = window.start;
    for range in ranges {
        let start = range.start.max(window.start);
        let end = range.end.min(window.end);
        if start >= end {
            continue;
        }
        pieces.push((cursor..start, false));
        pieces.push((start..end, true));
        cursor = end;
    }
    pieces.push((cursor..window.end, false));

    let mut segments: Vec<Segment> = Vec::with_capacity(pieces.len());
    // Whitespace seen since the last visible character. Matches never contain
    // whitespace, so a pending space always belongs to plain text.
    let mut pending_space = false;
    let mut started = false;
    for (range, hit) in pieces {
        let mut visible = String::with_capacity(range.len());
        for character in text[range].chars() {
            if character.is_whitespace() {
                pending_space = started;
                continue;
            }
            if pending_space && !hit {
                visible.push(' ');
                pending_space = false;
            }
            visible.push(character);
            started = true;
        }
        if hit && pending_space {
            // The space preceded this match; keep it with the plain text before it.
            match segments.last_mut() {
                Some(previous) if !previous.hit => previous.text.push(' '),
                Some(_) => segments.push(Segment {
                    text: " ".to_owned(),
                    hit: false,
                }),
                None => {}
            }
            pending_space = false;
        }
        if !visible.is_empty() {
            segments.push(Segment { text: visible, hit });
        }
    }
    segments
}

fn back_by_chars(text: &str, from: usize, count: usize) -> usize {
    text[..from]
        .char_indices()
        .rev()
        .nth(count.saturating_sub(1))
        .map_or(0, |(index, _)| index)
}

fn forward_by_chars(text: &str, from: usize, count: usize) -> usize {
    text[from..]
        .char_indices()
        .nth(count)
        .map_or(text.len(), |(index, _)| from + index)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clauses(needle: &str) -> Vec<Clause> {
        search_text::query_clauses(needle)
    }

    fn marked(segments: &[Segment]) -> String {
        segments
            .iter()
            .map(|segment| {
                if segment.hit {
                    format!("[{}]", segment.text)
                } else {
                    segment.text.clone()
                }
            })
            .collect()
    }

    #[test]
    fn marks_stems_inside_particle_forms() {
        let text = "김정은동지께서 조선로동당 대회에서 연설하시였다";
        assert_eq!(
            marked(&segments(text, &clauses("김정은"))),
            "[김정은]동지께서 조선로동당 대회에서 연설하시였다"
        );
        assert_eq!(
            marked(&segments(text, &clauses("로동당 연설"))),
            "김정은동지께서 조선[로동당] 대회에서 [연설]하시였다"
        );
    }

    #[test]
    fn marks_words_and_single_characters() {
        assert_eq!(
            marked(&segments("Peace talks; peaceful PEACE.", &clauses("peace"))),
            "[Peace] talks; peaceful [PEACE]."
        );
        assert_eq!(
            marked(&segments("북핵과 비핵화, 핵", &clauses("핵"))),
            "북[핵]과 비[핵]화, [핵]"
        );
        assert_eq!(marked(&segments("평화", &clauses("핵"))), "평화");
        assert!(segments("", &clauses("핵")).is_empty());
    }

    #[test]
    fn keeps_the_space_between_adjacent_matches_unmarked() {
        assert_eq!(
            marked(&segments("  평화\n\n통일 ", &clauses("평화 통일"))),
            "[평화] [통일]"
        );
    }

    #[test]
    fn merges_overlapping_matches() {
        assert_eq!(
            marked(&segments("조선로동당", &clauses("조선로 선로동"))),
            "[조선로동]당"
        );
    }

    #[test]
    fn snippet_centres_on_the_first_match_with_ellipses() {
        let text = format!(
            "{} 핵무력 강화 {}",
            "가나다라 ".repeat(40),
            "마바사아 ".repeat(60)
        );
        let cut = snippet(&text, &clauses("핵무력"));
        let rendered = marked(&cut);
        assert!(rendered.starts_with('…'));
        assert!(rendered.ends_with('…'));
        assert!(rendered.contains("[핵무력] 강화"));
        let visible = cut
            .iter()
            .map(|part| part.text.chars().count())
            .sum::<usize>();
        assert!(visible <= SNIPPET_CHARS + 2);
    }

    #[test]
    fn snippet_without_a_match_is_the_leading_text() {
        let cut = snippet("짧은 본문이다.\n\n둘째 문단.", &clauses("없는말"));
        assert_eq!(marked(&cut), "짧은 본문이다. 둘째 문단.");
        let long = "가".repeat(SNIPPET_CHARS + 50);
        let cut = snippet(&long, &[]);
        assert_eq!(cut.len(), 2);
        assert_eq!(cut[0].text.chars().count(), SNIPPET_CHARS);
        assert_eq!(cut[1].text, "…");
    }
}
