//! Query execution over the index, whether mapped from disk or held in memory.

use std::path::Path;

use tantivy::{
    DocAddress, Index, IndexReader, Order, ReloadPolicy, Searcher, TantivyDocument,
    collector::{Count, TopDocs},
};

use crate::{
    error::{Result, search_error},
    search::{
        collectors::{FacetCollector, FacetSelection, HistogramCollector},
        fields::{
            self, DATE, Fields, language_code, language_from_code, source_code, source_from_code,
        },
        query::{self, Scope},
        request::{Bucket, Facets, Filters, Histogram, Hit, Page, Sort},
        stored,
    },
};

pub struct SearchEngine {
    reader: IndexReader,
    fields: Fields,
}

impl SearchEngine {
    /// Opens the index on disk for reading. Commits made by an importer in
    /// another process become visible without reopening.
    pub fn open(directory: &Path) -> Result<Self> {
        let (index, fields) = fields::open_or_create(directory)?;
        Self::from_index(&index, fields, ReloadPolicy::OnCommitWithDelay)
    }

    pub fn from_index(index: &Index, fields: Fields, policy: ReloadPolicy) -> Result<Self> {
        let reader = index
            .reader_builder()
            .reload_policy(policy)
            .try_into()
            .map_err(search_error)?;
        Ok(Self { reader, fields })
    }

    pub(super) const fn fields(&self) -> &Fields {
        &self.fields
    }

    pub(super) fn searcher(&self) -> Searcher {
        self.reader.searcher()
    }

    /// Number of indexed documents.
    pub fn documents(&self) -> u64 {
        self.reader.searcher().num_docs()
    }

    /// Returns one page of matches and the total match count.
    pub fn search(
        &self,
        filters: &Filters,
        sort: Sort,
        offset: usize,
        limit: usize,
    ) -> Result<Page> {
        let searcher = self.reader.searcher();
        let query = query::build(
            &self.fields,
            filters,
            Scope {
                dates: true,
                facets: true,
            },
        )?;
        // The collector reserves room for `offset + limit` hits up front, so
        // both are capped by the corpus size instead of trusting the request.
        let documents = usize::try_from(searcher.num_docs()).unwrap_or(usize::MAX);
        let offset = offset.min(documents);
        let limit = limit.min(documents - offset);
        if limit == 0 {
            let total = searcher.search(&query, &Count).map_err(search_error)?;
            return Ok(Page {
                total,
                hits: Vec::new(),
            });
        }
        let top = || TopDocs::with_limit(limit).and_offset(offset);
        // Without query text every score is equal, so relevance means newest.
        let by_date = |order| top().order_by_fast_field::<i64>(DATE, order);
        let (total, addresses): (usize, Vec<DocAddress>) = match (sort, filters.clauses.is_empty())
        {
            (Sort::Relevance, false) => {
                let (total, hits) = searcher
                    .search(&query, &(Count, top().order_by_score()))
                    .map_err(search_error)?;
                (
                    total,
                    hits.into_iter().map(|(_, address)| address).collect(),
                )
            }
            (Sort::Oldest, _) => {
                let (total, hits) = searcher
                    .search(&query, &(Count, by_date(Order::Asc)))
                    .map_err(search_error)?;
                (
                    total,
                    hits.into_iter().map(|(_, address)| address).collect(),
                )
            }
            (Sort::Newest, _) | (Sort::Relevance, true) => {
                let (total, hits) = searcher
                    .search(&query, &(Count, by_date(Order::Desc)))
                    .map_err(search_error)?;
                (
                    total,
                    hits.into_iter().map(|(_, address)| address).collect(),
                )
            }
        };
        let hits = addresses
            .into_iter()
            .map(|address| self.hit(&searcher, address))
            .collect::<Result<Vec<_>>>()?;
        Ok(Page { total, hits })
    }

    /// Counts matches per time bucket. Year and month buckets cover the whole
    /// time axis, so a selected range can be widened from the same chart; day
    /// buckets are only meaningful inside the selected range and keep it.
    pub fn histogram(&self, filters: &Filters, bucket: Bucket) -> Result<Histogram> {
        let searcher = self.reader.searcher();
        let scope = Scope {
            dates: bucket == Bucket::Day,
            facets: true,
        };
        let query = query::build(&self.fields, filters, scope)?;
        let counts = searcher
            .search(&query, &HistogramCollector { bucket })
            .map_err(search_error)?;
        let mut histogram = counts
            .into_iter()
            .filter_map(|(key, count)| bucket.start(key).map(|start| (start, count)))
            .collect::<Vec<_>>();
        histogram.sort_unstable_by_key(|(start, _)| *start);
        Ok(histogram)
    }

    /// The oldest and newest publication dates in the index.
    pub fn coverage(&self) -> Result<Option<(time::Date, time::Date)>> {
        let all = Filters::default();
        let oldest = self.search(&all, Sort::Oldest, 0, 1)?.hits.pop();
        let newest = self.search(&all, Sort::Newest, 0, 1)?.hits.pop();
        Ok(oldest
            .zip(newest)
            .map(|(oldest, newest)| (oldest.date, newest.date)))
    }

    /// Counts matches per language, source, and category.
    pub fn facets(&self, filters: &Filters) -> Result<Facets> {
        let searcher = self.reader.searcher();
        let query = query::build(
            &self.fields,
            filters,
            Scope {
                dates: true,
                facets: false,
            },
        )?;
        let mask = |codes: Vec<u64>| {
            (!codes.is_empty())
                .then(|| codes.into_iter().fold(0_u64, |mask, code| mask | 1 << code))
        };
        let selection = FacetSelection {
            languages: mask(
                filters
                    .languages
                    .iter()
                    .map(|value| language_code(*value))
                    .collect(),
            ),
            sources: mask(
                filters
                    .sources
                    .iter()
                    .map(|value| source_code(*value))
                    .collect(),
            ),
            category: filters.category.clone(),
        };
        let counts = searcher
            .search(&query, &FacetCollector { selection })
            .map_err(search_error)?;
        let mut languages = counts
            .languages
            .into_iter()
            .filter_map(|(code, count)| language_from_code(code).map(|value| (value, count)))
            .collect::<Vec<_>>();
        languages.sort_unstable_by_key(|(value, _)| language_code(*value));
        let mut sources = counts
            .sources
            .into_iter()
            .filter_map(|(code, count)| source_from_code(code).map(|value| (value, count)))
            .collect::<Vec<_>>();
        sources.sort_unstable_by_key(|(value, _)| source_code(*value));
        let mut categories = counts.categories.into_iter().collect::<Vec<_>>();
        categories.sort_unstable_by(|left, right| {
            right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0))
        });
        Ok(Facets {
            languages,
            sources,
            categories,
        })
    }

    fn hit(&self, searcher: &Searcher, address: DocAddress) -> Result<Hit> {
        let document = searcher
            .doc::<TantivyDocument>(address)
            .map_err(search_error)?;
        stored::hit(&document, &self.fields)
    }
}
