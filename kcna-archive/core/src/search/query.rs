//! Translates request filters into an index query.

use std::ops::Bound;

use tantivy::{
    Term,
    query::{
        AllQuery, BooleanQuery, BoostQuery, Occur, PhraseQuery, Query, RangeQuery, RegexQuery,
        TermQuery,
    },
    schema::{Field, IndexRecordOption},
};

use crate::{
    error::{Error, Result, search_error},
    search::{
        fields::{Fields, date_value, language_code, source_code},
        request::Filters,
    },
    search_text::Clause,
};

const TITLE_BOOST: f32 = 3.0;

/// Which filters a query applies. The histogram shows the whole time axis and
/// the facets count across their own values, so each leaves one part out.
#[derive(Clone, Copy)]
pub struct Scope {
    pub dates: bool,
    pub facets: bool,
}

/// Builds the conjunction of the text clauses and the filters in `scope`.
pub fn build(fields: &Fields, filters: &Filters, scope: Scope) -> Result<Box<dyn Query>> {
    let mut conditions: Vec<(Occur, Box<dyn Query>)> = Vec::new();
    for clause in &filters.clauses {
        conditions.push((Occur::Must, clause_query(fields, clause)?));
    }
    if scope.facets {
        let any_of = |terms: Vec<Term>| -> Box<dyn Query> {
            Box::new(BooleanQuery::new(
                terms
                    .into_iter()
                    .map(|term| {
                        let query: Box<dyn Query> =
                            Box::new(TermQuery::new(term, IndexRecordOption::Basic));
                        (Occur::Should, query)
                    })
                    .collect(),
            ))
        };
        if !filters.languages.is_empty() {
            conditions.push((
                Occur::Must,
                any_of(
                    filters
                        .languages
                        .iter()
                        .map(|value| Term::from_field_u64(fields.language, language_code(*value)))
                        .collect(),
                ),
            ));
        }
        if !filters.sources.is_empty() {
            conditions.push((
                Occur::Must,
                any_of(
                    filters
                        .sources
                        .iter()
                        .map(|value| Term::from_field_u64(fields.source, source_code(*value)))
                        .collect(),
                ),
            ));
        }
        if let Some(category) = &filters.category {
            conditions.push((
                Occur::Must,
                Box::new(TermQuery::new(
                    Term::from_field_text(fields.category, category),
                    IndexRecordOption::Basic,
                )),
            ));
        }
    }
    if scope.dates && (filters.from.is_some() || filters.to.is_some()) {
        let bound = |date: Option<time::Date>| match date {
            Some(date) => Bound::Included(Term::from_field_i64(fields.date, date_value(date))),
            None => Bound::Unbounded,
        };
        conditions.push((
            Occur::Must,
            Box::new(RangeQuery::new(bound(filters.from), bound(filters.to))),
        ));
    }
    if conditions.is_empty() {
        return Ok(Box::new(AllQuery));
    }
    Ok(Box::new(BooleanQuery::new(conditions)))
}

/// One clause may match in the title or the body; title matches rank higher.
fn clause_query(fields: &Fields, clause: &Clause) -> Result<Box<dyn Query>> {
    let title = field_query(fields.title, clause)?;
    let body = field_query(fields.body, clause)?;
    let boosted: Box<dyn Query> = Box::new(BoostQuery::new(title, TITLE_BOOST));
    Ok(Box::new(BooleanQuery::new(vec![
        (Occur::Should, boosted),
        (Occur::Should, body),
    ])))
}

fn field_query(field: Field, clause: &Clause) -> Result<Box<dyn Query>> {
    let term = |lexeme: &str| Term::from_field_text(field, lexeme);
    match clause {
        Clause::Term(lexeme) => Ok(Box::new(TermQuery::new(
            term(lexeme),
            IndexRecordOption::WithFreqs,
        ))),
        Clause::Phrase(lexemes) => match lexemes.as_slice() {
            // Tantivy rejects a phrase of fewer than two terms.
            [] => Err(Error::Search("empty search phrase".to_owned())),
            [single] => Ok(Box::new(TermQuery::new(
                term(single),
                IndexRecordOption::WithFreqs,
            ))),
            many => Ok(Box::new(PhraseQuery::new(
                many.iter().map(|lexeme| term(lexeme)).collect(),
            ))),
        },
        Clause::Prefix(lexeme) => {
            // The analyzer only issues prefixes for a single CJK character,
            // which has no meaning in the pattern syntax.
            if !lexeme.chars().all(char::is_alphanumeric) {
                return Err(Error::Search("invalid search prefix".to_owned()));
            }
            RegexQuery::from_pattern(&format!("{lexeme}.*"), field)
                .map(|query| Box::new(query) as Box<dyn Query>)
                .map_err(search_error)
        }
    }
}
