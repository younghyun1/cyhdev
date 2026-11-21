//! Single-pass collectors for the histogram and the facet counts.

use std::collections::HashMap;

use tantivy::{
    DocId, Score, SegmentOrdinal, SegmentReader,
    collector::{Collector, SegmentCollector},
    columnar::{Column, StrColumn},
};

use crate::search::{
    fields::{CATEGORY, DATE, LANGUAGE, SOURCE, date_from_value},
    request::Bucket,
};

/// Counts matching documents per time bucket.
pub struct HistogramCollector {
    pub bucket: Bucket,
}

pub struct HistogramSegment {
    bucket: Bucket,
    dates: Column<i64>,
    counts: HashMap<i64, u64>,
    /// The bucket of the previous document: neighbours usually share one, so
    /// this avoids recomputing the calendar conversion per document.
    last: Option<(i64, i64)>,
}

impl Collector for HistogramCollector {
    type Fruit = HashMap<i64, u64>;
    type Child = HistogramSegment;

    fn for_segment(
        &self,
        _: SegmentOrdinal,
        segment: &SegmentReader,
    ) -> tantivy::Result<Self::Child> {
        Ok(HistogramSegment {
            bucket: self.bucket,
            dates: segment.fast_fields().i64(DATE)?,
            counts: HashMap::new(),
            last: None,
        })
    }

    fn requires_scoring(&self) -> bool {
        false
    }

    fn merge_fruits(&self, fruits: Vec<Self::Fruit>) -> tantivy::Result<Self::Fruit> {
        let mut merged = HashMap::new();
        for fruit in fruits {
            for (key, count) in fruit {
                *merged.entry(key).or_insert(0) += count;
            }
        }
        Ok(merged)
    }
}

impl SegmentCollector for HistogramSegment {
    type Fruit = HashMap<i64, u64>;

    fn collect(&mut self, doc: DocId, _: Score) {
        let Some(value) = self.dates.first(doc) else {
            return;
        };
        let key = match self.last {
            Some((seen, key)) if seen == value => key,
            _ => {
                let Some(date) = date_from_value(value) else {
                    return;
                };
                let key = self.bucket.key(date);
                self.last = Some((value, key));
                key
            }
        };
        *self.counts.entry(key).or_insert(0) += 1;
    }

    fn harvest(self) -> Self::Fruit {
        self.counts
    }
}

/// Selected filter values as the collector checks them per document.
#[derive(Clone, Debug, Default)]
pub struct FacetSelection {
    /// Bit `n` set selects code `n`; `None` selects every value.
    pub languages: Option<u64>,
    pub sources: Option<u64>,
    pub category: Option<String>,
}

/// Counts per language, source, and category over the documents matching the
/// text and date range. Each group applies the other two selections but not
/// its own.
pub struct FacetCollector {
    pub selection: FacetSelection,
}

#[derive(Debug, Default)]
pub struct FacetCounts {
    pub languages: HashMap<u64, u64>,
    pub sources: HashMap<u64, u64>,
    pub categories: HashMap<String, u64>,
}

pub struct FacetSegment {
    languages: Column<u64>,
    sources: Column<u64>,
    categories: Option<StrColumn>,
    language_mask: Option<u64>,
    source_mask: Option<u64>,
    /// `Some(None)` when a category is selected that this segment lacks.
    category_ordinal: Option<Option<u64>>,
    language_counts: [u64; 64],
    source_counts: [u64; 64],
    category_counts: Vec<u64>,
}

impl Collector for FacetCollector {
    type Fruit = FacetCounts;
    type Child = FacetSegment;

    fn for_segment(
        &self,
        _: SegmentOrdinal,
        segment: &SegmentReader,
    ) -> tantivy::Result<Self::Child> {
        let fast = segment.fast_fields();
        let categories = fast.str(CATEGORY)?;
        let category_ordinal = match (&self.selection.category, &categories) {
            (None, _) => None,
            (Some(_), None) => Some(None),
            (Some(name), Some(column)) => Some(column.dictionary().term_ord(name.as_bytes())?),
        };
        let terms = categories.as_ref().map_or(0, |column| column.num_terms());
        Ok(FacetSegment {
            languages: fast.u64(LANGUAGE)?,
            sources: fast.u64(SOURCE)?,
            categories,
            language_mask: self.selection.languages,
            source_mask: self.selection.sources,
            category_ordinal,
            language_counts: [0; 64],
            source_counts: [0; 64],
            category_counts: vec![0; terms],
        })
    }

    fn requires_scoring(&self) -> bool {
        false
    }

    fn merge_fruits(&self, fruits: Vec<Self::Fruit>) -> tantivy::Result<Self::Fruit> {
        let mut merged = FacetCounts::default();
        for fruit in fruits {
            for (code, count) in fruit.languages {
                *merged.languages.entry(code).or_insert(0) += count;
            }
            for (code, count) in fruit.sources {
                *merged.sources.entry(code).or_insert(0) += count;
            }
            for (name, count) in fruit.categories {
                *merged.categories.entry(name).or_insert(0) += count;
            }
        }
        Ok(merged)
    }
}

impl SegmentCollector for FacetSegment {
    type Fruit = FacetCounts;

    fn collect(&mut self, doc: DocId, _: Score) {
        let language = self.languages.first(doc).filter(|code| *code < 64);
        // An article held by several sources carries one value per source.
        let sources = self
            .sources
            .values_for_doc(doc)
            .filter(|code| *code < 64)
            .fold(0_u64, |mask, code| mask | 1 << code);
        let category = self
            .categories
            .as_ref()
            .and_then(|column| column.term_ords(doc).next());
        let language_ok = match (self.language_mask, language) {
            (None, _) => true,
            (Some(mask), Some(code)) => mask & (1 << code) != 0,
            (Some(_), None) => false,
        };
        let source_ok = self.source_mask.is_none_or(|mask| mask & sources != 0);
        let category_ok = match self.category_ordinal {
            None => true,
            Some(wanted) => wanted.is_some() && wanted == category,
        };
        if let Some(code) = language
            && source_ok
            && category_ok
        {
            self.language_counts[code as usize] += 1;
        }
        if language_ok && category_ok {
            for (code, count) in self.source_counts.iter_mut().enumerate() {
                if sources & (1 << code) != 0 {
                    *count += 1;
                }
            }
        }
        if let Some(ordinal) = category
            && language_ok
            && source_ok
            && let Some(count) = usize::try_from(ordinal)
                .ok()
                .and_then(|index| self.category_counts.get_mut(index))
        {
            *count += 1;
        }
    }

    fn harvest(self) -> Self::Fruit {
        let codes = |counts: [u64; 64]| {
            counts
                .into_iter()
                .enumerate()
                .filter(|(_, count)| *count > 0)
                .map(|(code, count)| (code as u64, count))
                .collect::<HashMap<_, _>>()
        };
        let mut categories = HashMap::new();
        if let Some(column) = &self.categories {
            let mut name = String::new();
            for (ordinal, count) in self.category_counts.iter().enumerate() {
                if *count == 0 {
                    continue;
                }
                name.clear();
                // An ordinal the dictionary cannot resolve is skipped; the
                // counts for the remaining categories are still correct.
                if matches!(column.ord_to_str(ordinal as u64, &mut name), Ok(true)) {
                    categories.insert(name.clone(), *count);
                }
            }
        }
        FacetCounts {
            languages: codes(self.language_counts),
            sources: codes(self.source_counts),
            categories,
        }
    }
}
