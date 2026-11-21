//! Tantivy tokenizer backed by the shared search analyzer.

use tantivy::tokenizer::{Token, TokenStream, Tokenizer};

use crate::search_text;

/// Feeds Tantivy the lexemes of the shared analyzer, so memory mode matches
/// the same text as the PostgreSQL search vector.
#[derive(Clone, Default)]
pub struct ArchiveTokenizer;

pub struct ArchiveTokenStream {
    pending: std::vec::IntoIter<Token>,
    current: Token,
}

impl Tokenizer for ArchiveTokenizer {
    type TokenStream<'a> = ArchiveTokenStream;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        let mut tokens = Vec::new();
        search_text::document_lexemes(text, |lexeme| {
            tokens.push(Token {
                // Byte offsets serve highlighting, which this index does not do.
                offset_from: 0,
                offset_to: 0,
                position: tokens.len(),
                text: lexeme.to_owned(),
                position_length: 1,
            });
        });
        ArchiveTokenStream {
            pending: tokens.into_iter(),
            current: Token::default(),
        }
    }
}

impl TokenStream for ArchiveTokenStream {
    fn advance(&mut self) -> bool {
        match self.pending.next() {
            Some(token) => {
                self.current = token;
                true
            }
            None => false,
        }
    }

    fn token(&self) -> &Token {
        &self.current
    }

    fn token_mut(&mut self) -> &mut Token {
        &mut self.current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streams_lexemes_with_consecutive_positions() {
        let mut tokenizer = ArchiveTokenizer;
        let mut stream = tokenizer.token_stream("김정은 KCNA");
        let mut seen = Vec::new();
        while stream.advance() {
            seen.push((stream.token().text.clone(), stream.token().position));
        }
        let expected = [("김정", 0), ("정은", 1), ("은", 2), ("kcna", 3)];
        assert_eq!(
            seen,
            expected.map(|(text, position)| (text.to_owned(), position))
        );
    }
}
