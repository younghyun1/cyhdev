//! Tests for query execution, against an in-memory index with the production layout.

use tantivy::ReloadPolicy;
use time::{Date, macros::date};
use uuid::Uuid;

use crate::{
    error::{Result, search_error},
    search::{
        engine::SearchEngine,
        fields,
        request::{Bucket, Filters, Sort},
        writer::{self, IndexedDocument},
    },
    search_text,
    types::{Language, PublicationPrecision, Source},
};

fn entry(
    id: u128,
    day: Date,
    language: Language,
    source: Source,
    category: Option<&str>,
    title: &str,
    body: &str,
) -> IndexedDocument {
    IndexedDocument {
        id: Uuid::from_u128(id),
        title: title.to_owned(),
        body: (!body.is_empty()).then(|| body.to_owned()),
        date: day,
        precision: PublicationPrecision::Day,
        language,
        source,
        canonical_url: format!("http://example.invalid/{id}"),
        category: category.map(str::to_owned),
        source_record_id: None,
        metadata: serde_json::Value::Null,
        translation_group: None,
        copies: Vec::new(),
    }
}

pub(crate) fn corpus() -> Vec<IndexedDocument> {
    use Language::{Chinese, English, Korean};
    use Source::{KcnaCoJp, KcnaKp, StilesArchive};
    vec![
        entry(
            1,
            date!(2026 - 11 - 20),
            Korean,
            KcnaKp,
            Some("국내소식"),
            "김정은동지께서 연설",
            "조선로동당 제8차대회에서 핵무력 강화를 강조하시였다",
        ),
        entry(
            2,
            date!(2026 - 11 - 05),
            Korean,
            KcnaKp,
            Some("국제소식"),
            "평화 성명",
            "외무성 대변인 담화",
        ),
        entry(
            3,
            date!(2026 - 10 - 10),
            English,
            KcnaKp,
            Some("Home News"),
            "Peace statement",
            "Diplomatic note about talks",
        ),
        entry(
            4,
            date!(2025 - 06 - 01),
            English,
            KcnaCoJp,
            None,
            "Diplomatic note",
            "A statement about peace talks",
        ),
        entry(
            5,
            date!(2024 - 01 - 15),
            English,
            StilesArchive,
            None,
            "Headline only",
            "",
        ),
        entry(
            6,
            date!(2003 - 05 - 31),
            Chinese,
            KcnaCoJp,
            None,
            "报道",
            "朝鲜劳动党总书记金正恩同志",
        ),
    ]
}

/// Builds an in-memory index over `documents` with the production layout.
pub(crate) fn engine_over(documents: &[IndexedDocument]) -> Result<SearchEngine> {
    let (index, layout) = fields::create_in_ram()?;
    let mut writer = index
        .writer_with_num_threads(1, 20_000_000)
        .map_err(search_error)?;
    for document in documents {
        writer
            .add_document(writer::document(layout, document))
            .map_err(search_error)?;
    }
    writer.commit().map_err(search_error)?;
    SearchEngine::from_index(&index, layout, ReloadPolicy::Manual)
}

fn engine() -> Result<SearchEngine> {
    engine_over(&corpus())
}

fn filters(text: &str) -> Filters {
    Filters {
        clauses: search_text::query_clauses(text),
        ..Filters::default()
    }
}

fn ids(engine: &SearchEngine, filters: &Filters, sort: Sort) -> Result<Vec<u128>> {
    Ok(engine
        .search(filters, sort, 0, 10)?
        .hits
        .into_iter()
        .map(|hit| hit.id.as_u128())
        .collect())
}

#[test]
fn browses_everything_newest_first_without_text() -> Result<()> {
    let engine = engine()?;
    assert_eq!(engine.documents(), 6);
    assert_eq!(
        ids(&engine, &Filters::default(), Sort::Relevance)?,
        [1, 2, 3, 4, 5, 6]
    );
    assert_eq!(
        ids(&engine, &Filters::default(), Sort::Oldest)?,
        [6, 5, 4, 3, 2, 1]
    );
    let page = engine.search(&Filters::default(), Sort::Newest, 4, 10)?;
    assert_eq!((page.total, page.hits.len()), (6, 2));
    let beyond = engine.search(&Filters::default(), Sort::Newest, usize::MAX, usize::MAX)?;
    assert_eq!((beyond.total, beyond.hits.len()), (6, 0));
    Ok(())
}

#[test]
fn matches_stems_phrases_and_single_characters() -> Result<()> {
    let engine = engine()?;
    assert_eq!(ids(&engine, &filters("김정은"), Sort::Relevance)?, [1]);
    assert_eq!(ids(&engine, &filters("로동당 대회"), Sort::Relevance)?, [1]);
    assert_eq!(ids(&engine, &filters("핵"), Sort::Relevance)?, [1]);
    assert_eq!(ids(&engine, &filters("劳动党"), Sort::Relevance)?, [6]);
    assert!(ids(&engine, &filters("김정일"), Sort::Relevance)?.is_empty());
    // A title match outranks a body match.
    assert_eq!(ids(&engine, &filters("peace"), Sort::Relevance)?, [3, 4]);
    assert_eq!(ids(&engine, &filters("peace"), Sort::Oldest)?, [4, 3]);
    Ok(())
}

#[test]
fn applies_language_source_category_and_date_filters() -> Result<()> {
    let engine = engine()?;
    let mut request = Filters {
        languages: vec![Language::English],
        ..Filters::default()
    };
    assert_eq!(ids(&engine, &request, Sort::Newest)?, [3, 4, 5]);
    request.sources = vec![Source::KcnaCoJp, Source::StilesArchive];
    assert_eq!(ids(&engine, &request, Sort::Newest)?, [4, 5]);
    request.from = Some(date!(2025 - 01 - 01));
    assert_eq!(ids(&engine, &request, Sort::Newest)?, [4]);
    let category = Filters {
        category: Some("국내소식".to_owned()),
        ..Filters::default()
    };
    assert_eq!(ids(&engine, &category, Sort::Newest)?, [1]);
    let range = Filters {
        from: Some(date!(2026 - 11 - 05)),
        to: Some(date!(2026 - 11 - 05)),
        ..Filters::default()
    };
    assert_eq!(ids(&engine, &range, Sort::Newest)?, [2]);
    Ok(())
}

#[test]
fn returns_stored_fields_for_a_hit() -> Result<()> {
    let engine = engine()?;
    let page = engine.search(&filters("Headline"), Sort::Relevance, 0, 10)?;
    let hit = page.hits.first();
    assert!(hit.is_some_and(|hit| {
        hit.title == "Headline only"
            && hit.body.is_none()
            && hit.date == date!(2024 - 01 - 15)
            && hit.language == Language::English
            && hit.sources == [Source::StilesArchive]
            && hit.category.is_none()
    }));
    Ok(())
}

#[test]
fn histogram_covers_the_whole_axis_except_for_day_buckets() -> Result<()> {
    let engine = engine()?;
    let november = Filters {
        from: Some(date!(2026 - 11 - 01)),
        to: Some(date!(2026 - 11 - 30)),
        ..Filters::default()
    };
    let months = engine.histogram(&november, Bucket::Month)?;
    assert_eq!(months.len(), 5);
    assert_eq!(months.first(), Some(&(date!(2003 - 05 - 01), 1)));
    assert_eq!(months.last(), Some(&(date!(2026 - 11 - 01), 2)));
    let years = engine.histogram(&filters("peace"), Bucket::Year)?;
    assert_eq!(
        years,
        [(date!(2025 - 01 - 01), 1), (date!(2026 - 01 - 01), 1)]
    );
    let days = engine.histogram(&november, Bucket::Day)?;
    assert_eq!(
        days,
        [(date!(2026 - 11 - 05), 1), (date!(2026 - 11 - 20), 1)]
    );
    Ok(())
}

#[test]
fn facets_count_across_their_own_filter() -> Result<()> {
    let engine = engine()?;
    let english = Filters {
        languages: vec![Language::English],
        ..Filters::default()
    };
    let facets = engine.facets(&english)?;
    // Languages ignore the language filter; sources and categories apply it.
    assert_eq!(
        facets.languages,
        [
            (Language::Korean, 2),
            (Language::English, 3),
            (Language::Chinese, 1)
        ]
    );
    assert_eq!(
        facets.sources,
        [
            (Source::KcnaKp, 1),
            (Source::KcnaCoJp, 1),
            (Source::StilesArchive, 1)
        ]
    );
    assert_eq!(facets.categories, [("Home News".to_owned(), 1)]);
    let home = Filters {
        category: Some("국내소식".to_owned()),
        ..Filters::default()
    };
    let facets = engine.facets(&home)?;
    assert_eq!(facets.languages, [(Language::Korean, 1)]);
    assert_eq!(facets.categories.len(), 3);
    Ok(())
}
