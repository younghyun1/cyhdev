//! Tests for single-document reads: copies, aliases, and translation groups.

use time::macros::date;
use uuid::Uuid;

use crate::{
    error::Result,
    search::{engine_tests::engine_over, request::CopyRef, writer::IndexedDocument},
    types::{Language, PublicationPrecision, Source},
};

fn article(id: u128, language: Language, group: Option<u128>) -> IndexedDocument {
    IndexedDocument {
        id: Uuid::from_u128(id),
        title: format!("title {id}"),
        body: Some("first paragraph\n\nsecond paragraph".to_owned()),
        date: date!(2026 - 11 - 21),
        precision: PublicationPrecision::Minute,
        language,
        source: Source::KcnaKp,
        canonical_url: format!("http://example.invalid/{id}"),
        category: Some("Home News".to_owned()),
        source_record_id: Some(format!("record-{id}")),
        metadata: serde_json::json!({ "page": id }),
        translation_group: group.map(Uuid::from_u128),
        copies: Vec::new(),
    }
}

#[test]
fn reads_every_stored_field_back() -> Result<()> {
    let mut korean = article(1, Language::Korean, Some(900));
    korean.copies = vec![CopyRef {
        id: Uuid::from_u128(11),
        source: Source::KcnaCoJp,
        url: "http://example.invalid/copy".to_owned(),
    }];
    let engine = engine_over(&[korean.clone()])?;
    let stored = engine.document(Uuid::from_u128(1))?;
    assert!(stored.is_some_and(|stored| {
        stored.title == "title 1"
            && stored.precision == PublicationPrecision::Minute
            && stored.sources == [Source::KcnaKp, Source::KcnaCoJp]
            && stored.canonical_url == "http://example.invalid/1"
            && stored.source_record_id.as_deref() == Some("record-1")
            && stored.metadata["page"] == 1
            && stored.translation_group == Some(Uuid::from_u128(900))
            && stored.copies == korean.copies
    }));
    assert!(engine.document(Uuid::from_u128(404))?.is_none());
    Ok(())
}

#[test]
fn a_copy_identifier_opens_the_preferred_document() -> Result<()> {
    let mut preferred = article(1, Language::Korean, None);
    preferred.copies = vec![CopyRef {
        id: Uuid::from_u128(11),
        source: Source::KcnaCoJp,
        url: "http://example.invalid/copy".to_owned(),
    }];
    let engine = engine_over(&[preferred, article(2, Language::Korean, None)])?;
    let through_copy = engine.document(Uuid::from_u128(11))?;
    assert!(through_copy.is_some_and(|stored| stored.id == Uuid::from_u128(1)));
    Ok(())
}

#[test]
fn lists_other_editions_once_per_language_in_code_order() -> Result<()> {
    let engine = engine_over(&[
        article(1, Language::Korean, Some(900)),
        article(2, Language::Russian, Some(900)),
        article(3, Language::English, Some(900)),
        article(4, Language::English, Some(900)),
        article(5, Language::Chinese, Some(901)),
        article(6, Language::Japanese, None),
    ])?;
    let Some(korean) = engine.document(Uuid::from_u128(1))? else {
        return Err(crate::error::Error::Search(
            "document 1 is missing".to_owned(),
        ));
    };
    let editions = engine.translations(&korean)?;
    let languages = editions
        .iter()
        .map(|edition| edition.language)
        .collect::<Vec<_>>();
    // Codes sort as eng, rus; the second English member is not repeated and
    // the other group is not included.
    assert_eq!(languages, [Language::English, Language::Russian]);
    let Some(alone) = engine.document(Uuid::from_u128(6))? else {
        return Err(crate::error::Error::Search(
            "document 6 is missing".to_owned(),
        ));
    };
    assert!(engine.translations(&alone)?.is_empty());
    Ok(())
}
