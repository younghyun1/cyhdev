//! Script-aware text analysis shared by the PostgreSQL and in-memory search backends.
//!
//! Whitespace tokenization cannot search this corpus. Korean attaches particles
//! to stems, so `김정은` never equals the token `김정은동지께서`, and Chinese and
//! Japanese have no word spaces at all. Both backends therefore index the same
//! lexemes produced here:
//!
//! - a run of non-CJK letters or digits is one lowercased word;
//! - a run of CJK characters becomes overlapping bigrams, followed by its last
//!   character alone.
//!
//! The trailing single character means every CJK character starts exactly one
//! lexeme: a bigram when another character follows, the single otherwise. A
//! multi-character query is then a phrase of consecutive bigrams, which matches
//! any substring, and a one-character query is a prefix match over lexemes.

use std::ops::Range;

/// Words longer than this are identifiers or markup debris, not search terms.
const MAX_WORD_CHARS: usize = 64;

/// One searchable unit of a query; clauses are combined conjunctively.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Clause {
    /// Exact lexeme.
    Term(String),
    /// Lexemes at consecutive positions.
    Phrase(Vec<String>),
    /// Every lexeme starting with this single CJK character.
    Prefix(String),
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum Class {
    Cjk,
    Word,
    Break,
}

/// Calls `emit` with each lexeme of `text` in document order.
///
/// A lexeme's position is its index in this sequence, which is what makes a
/// query phrase of bigrams line up with the indexed text.
pub fn document_lexemes(text: &str, mut emit: impl FnMut(&str)) {
    document_spans(text, |lexeme, _| emit(lexeme));
}

/// Like [`document_lexemes`], and also reports the byte range of `text` each
/// lexeme was read from, for marking matches in displayed text.
pub fn document_spans(text: &str, mut emit: impl FnMut(&str, Range<usize>)) {
    for_each_run(text, |class, start, run| match class {
        Class::Word => {
            if let Some(word) = word(run) {
                emit(&word, start..start + run.len());
            }
        }
        Class::Cjk => {
            let mut bigram = String::with_capacity(8);
            let mut previous: Option<(usize, char)> = None;
            for (offset, character) in run.char_indices() {
                if let Some((first_offset, first)) = previous {
                    bigram.clear();
                    bigram.push(first);
                    bigram.push(character);
                    let end = start + offset + character.len_utf8();
                    emit(&bigram, start + first_offset..end);
                }
                previous = Some((offset, character));
            }
            if let Some((offset, last)) = previous {
                let from = start + offset;
                emit(last.encode_utf8(&mut [0; 4]), from..from + last.len_utf8());
            }
        }
        Class::Break => {}
    });
}

/// Splits a user query into conjunctive clauses matching [`document_lexemes`].
pub fn query_clauses(needle: &str) -> Vec<Clause> {
    let mut clauses = Vec::new();
    for_each_run(needle, |class, _, run| match class {
        Class::Word => {
            if let Some(word) = word(run) {
                clauses.push(Clause::Term(word));
            }
        }
        Class::Cjk => {
            let characters = run.chars().collect::<Vec<_>>();
            match characters.as_slice() {
                [] => {}
                [single] => clauses.push(Clause::Prefix(single.to_string())),
                [first, second] => clauses.push(Clause::Term([*first, *second].iter().collect())),
                many => clauses.push(Clause::Phrase(
                    many.windows(2)
                        .map(|pair| pair.iter().collect::<String>())
                        .collect(),
                )),
            }
        }
        Class::Break => {}
    });
    clauses
}

fn word(run: &str) -> Option<String> {
    (run.chars().count() <= MAX_WORD_CHARS).then(|| run.to_lowercase())
}

/// Visits each maximal run of one class with its byte offset in `text`.
fn for_each_run(text: &str, mut visit: impl FnMut(Class, usize, &str)) {
    let mut start = 0;
    let mut current = Class::Break;
    for (index, character) in text.char_indices() {
        let class = classify(character);
        if class != current {
            if current != Class::Break {
                visit(current, start, &text[start..index]);
            }
            start = index;
            current = class;
        }
    }
    if current != Class::Break {
        visit(current, start, &text[start..]);
    }
}

fn classify(character: char) -> Class {
    if is_cjk(character) {
        Class::Cjk
    } else if character.is_alphanumeric() {
        Class::Word
    } else {
        Class::Break
    }
}

/// Scripts written without spaces between words, plus Hangul, whose spacing
/// units carry attached particles.
const fn is_cjk(character: char) -> bool {
    matches!(
        character as u32,
        0x1100..=0x11ff      // Hangul Jamo
            | 0x2e80..=0x2fdf  // CJK and Kangxi radicals
            | 0x3005..=0x3007  // iteration mark, closing mark, ideographic zero
            | 0x3040..=0x30ff  // Hiragana and Katakana
            | 0x3130..=0x318f  // Hangul compatibility Jamo
            | 0x31f0..=0x31ff  // Katakana phonetic extensions
            | 0x3400..=0x4dbf  // CJK extension A
            | 0x4e00..=0x9fff  // CJK unified ideographs
            | 0xa960..=0xa97f  // Hangul Jamo extended A
            | 0xac00..=0xd7ff  // Hangul syllables and Jamo extended B
            | 0xf900..=0xfaff  // CJK compatibility ideographs
            | 0xff66..=0xff9f  // halfwidth Katakana
            | 0x20000..=0x3ffff // CJK extensions B and later
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lexemes(text: &str) -> Vec<String> {
        let mut collected = Vec::new();
        document_lexemes(text, |lexeme| collected.push(lexeme.to_owned()));
        collected
    }

    /// Reference matcher: a clause holds when its lexemes occur at consecutive
    /// positions, or when any lexeme has the prefix.
    fn matches(text: &str, needle: &str) -> bool {
        let document = lexemes(text);
        let clauses = query_clauses(needle);
        !clauses.is_empty()
            && clauses.iter().all(|clause| match clause {
                Clause::Term(term) => document.contains(term),
                Clause::Phrase(phrase) => document
                    .windows(phrase.len())
                    .any(|window| window == phrase.as_slice()),
                Clause::Prefix(prefix) => document.iter().any(|lexeme| lexeme.starts_with(prefix)),
            })
    }

    #[test]
    fn splits_cjk_runs_into_bigrams_with_a_trailing_single() {
        assert_eq!(
            lexemes("김정은동지"),
            ["김정", "정은", "은동", "동지", "지"]
        );
        assert_eq!(lexemes("핵"), ["핵"]);
        assert_eq!(lexemes("Peace Talks, 2026!"), ["peace", "talks", "2026"]);
        assert_eq!(lexemes("제8차 대회"), ["제", "8", "차", "대회", "회"]);
    }

    #[test]
    fn finds_korean_stems_inside_particle_forms() {
        let text = "김정은동지께서 조선로동당 제8차대회에서 연설하시였다";
        assert!(matches(text, "김정은"));
        assert!(matches(text, "로동당"));
        assert!(matches(text, "연설"));
        assert!(matches(text, "대회 김정은"));
        assert!(!matches(text, "김정일"));
        assert!(!matches(text, "정김"));
    }

    #[test]
    fn finds_unsegmented_chinese_and_japanese() {
        assert!(matches("朝鲜劳动党总书记金正恩同志", "劳动党"));
        assert!(matches("朝鮮労働党の金正恩総書記は", "金正恩"));
        assert!(matches("朝鮮労働党の金正恩総書記は", "の"));
        assert!(!matches("朝鲜劳动党总书记", "书记处"));
    }

    #[test]
    fn single_characters_match_at_any_position_in_a_run() {
        assert!(matches("북핵 문제", "핵"));
        assert!(matches("핵무력", "핵"));
        assert!(matches("비핵화", "핵"));
        assert!(!matches("평화", "핵"));
    }

    #[test]
    fn words_match_whole_and_case_insensitively() {
        assert!(matches("Pyongyang, January 1 (KCNA)", "kcna pyongyang"));
        assert!(matches("Заявление МИД КНДР", "кндр"));
        assert!(!matches("peaceful", "peace"));
        assert!(query_clauses("  ,.!  ").is_empty());
    }

    #[test]
    fn spans_cover_the_text_each_lexeme_came_from() {
        let text = "KCNA 김정은, 핵";
        let mut spans = Vec::new();
        document_spans(text, |lexeme, range| {
            spans.push((lexeme.to_owned(), &text[range]))
        });
        let expected = [
            ("kcna", "KCNA"),
            ("김정", "김정"),
            ("정은", "정은"),
            ("은", "은"),
            ("핵", "핵"),
        ];
        assert_eq!(
            spans,
            expected.map(|(lexeme, source)| (lexeme.to_owned(), source))
        );
    }

    #[test]
    fn drops_overlong_words() {
        let debris = "a".repeat(MAX_WORD_CHARS + 1);
        assert!(lexemes(&debris).is_empty());
        assert_eq!(lexemes(&"a".repeat(MAX_WORD_CHARS)).len(), 1);
    }
}
