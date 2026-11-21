//! The archive service: one loaded snapshot and every read it answers.

use std::path::Path;

use tantivy::ReloadPolicy;

use crate::{
    error::Error,
    search::{
        engine::SearchEngine,
        fields, highlight,
        request::{Facets, Filters, Hit, Page},
    },
    search_text::Clause,
    service::{
        export::{self, ExportFormat},
        models::{
            FacetGroups, FacetValue, HistogramBucket, HistogramResponse, SearchItem,
            SearchResponse, StatusResponse, TextSegment, TrendsResponse,
        },
        params::QueryParams,
        trends,
    },
    snapshot::{layout::index_directory, manifest::Manifest, ram},
};

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("invalid request")]
    BadRequest,
    #[error("not found")]
    NotFound,
    #[error(transparent)]
    Unavailable(#[from] Error),
}

/// A snapshot opened for reading.
///
/// Every method is synchronous and CPU-bound; an async host calls them from
/// its blocking pool.
pub struct Archive {
    engine: SearchEngine,
    manifest: Manifest,
    /// Bytes of index held in memory, when it was loaded there.
    memory_bytes: Option<u64>,
}

impl Archive {
    /// Reads the snapshot in `directory` fully into memory. Later requests
    /// touch neither the filesystem nor a database, and a snapshot replaced
    /// on disk is picked up by loading again.
    pub fn load_into_memory(directory: &Path) -> Result<Self, Error> {
        let manifest = Manifest::read(directory)?;
        let loaded = ram::load(&index_directory(directory))?;
        let layout = fields::adopt(&loaded.index)?;
        let engine = SearchEngine::from_index(&loaded.index, layout, ReloadPolicy::Manual)?;
        Ok(Self {
            engine,
            manifest,
            memory_bytes: Some(loaded.bytes),
        })
    }

    /// Opens the snapshot in `directory` through the page cache. Commits by a
    /// collector running beside it become visible without reopening.
    pub fn open_on_disk(directory: &Path) -> Result<Self, Error> {
        let manifest = Manifest::read(directory)?;
        let engine = SearchEngine::open(&index_directory(directory))?;
        Ok(Self {
            engine,
            manifest,
            memory_bytes: None,
        })
    }

    pub const fn engine(&self) -> &SearchEngine {
        &self.engine
    }

    pub const fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// Bytes of index held in memory, when it was loaded there.
    pub const fn memory_bytes(&self) -> Option<u64> {
        self.memory_bytes
    }

    /// One page of results with snippets and facet counts.
    pub fn search(&self, params: &QueryParams) -> Result<SearchResponse, ServiceError> {
        let filters = params.filters()?;
        let sort = params.sort()?;
        let (offset, limit) = params.window()?;
        if params.is_unsearchable(&filters) {
            return Ok(SearchResponse {
                total: 0,
                offset,
                limit,
                items: Vec::new(),
                facets: FacetGroups::default(),
            });
        }
        let Page { total, hits } = self.engine.search(&filters, sort, offset, limit)?;
        let facets = self.engine.facets(&filters)?;
        Ok(SearchResponse {
            total,
            offset,
            limit,
            items: hits
                .into_iter()
                .map(|hit| item(hit, &filters.clauses))
                .collect(),
            facets: facet_groups(facets),
        })
    }

    /// Matches per time bucket over the whole time axis.
    pub fn histogram(&self, params: &QueryParams) -> Result<HistogramResponse, ServiceError> {
        let filters = params.filters()?;
        let bucket = params.bucket(&filters)?;
        if params.is_unsearchable(&filters) {
            return Ok(HistogramResponse {
                bucket: bucket.name(),
                buckets: Vec::new(),
            });
        }
        Ok(HistogramResponse {
            bucket: bucket.name(),
            buckets: self
                .engine
                .histogram(&filters, bucket)?
                .into_iter()
                .map(|(start, count)| HistogramBucket {
                    start: start.to_string(),
                    count,
                })
                .collect(),
        })
    }

    /// Counts for up to four terms per time bucket.
    pub fn trends(
        &self,
        params: &QueryParams,
        terms: Vec<String>,
    ) -> Result<TrendsResponse, ServiceError> {
        trends::trends(&self.engine, params, terms)
    }

    /// A result set serialized as CSV or JSON Lines.
    pub fn export(
        &self,
        params: &QueryParams,
        format: ExportFormat,
    ) -> Result<String, ServiceError> {
        export::export(&self.engine, params, format)
    }

    /// Corpus and index summary.
    pub fn status(&self) -> Result<StatusResponse, ServiceError> {
        let coverage = self.engine.coverage()?;
        let facets = self.engine.facets(&Filters::default())?;
        Ok(StatusResponse {
            documents: self.manifest.documents,
            copies: self.manifest.copies,
            indexed: self.engine.documents(),
            translation_groups: self.manifest.translation_groups,
            oldest_date: coverage.map(|(oldest, _)| oldest.to_string()),
            newest_date: coverage.map(|(_, newest)| newest.to_string()),
            last_retrieved_at: self.manifest.last_retrieved_at.clone(),
            snapshot_built_at: self.manifest.built_at.clone(),
            memory_bytes: self.memory_bytes,
            languages: values(facets.languages),
            sources: values(facets.sources),
        })
    }
}

pub(super) fn segments(text: &str, clauses: &[Clause]) -> Vec<TextSegment> {
    highlight::segments(text, clauses)
        .into_iter()
        .map(TextSegment::from)
        .collect()
}

fn item(hit: Hit, clauses: &[Clause]) -> SearchItem {
    SearchItem {
        id: hit.id.to_string(),
        title: segments(&hit.title, clauses),
        snippet: hit
            .body
            .as_deref()
            .map(|body| highlight::snippet(body, clauses))
            .unwrap_or_default()
            .into_iter()
            .map(TextSegment::from)
            .collect(),
        publication_date: hit.date.to_string(),
        language: hit.language,
        sources: hit.sources,
        category: hit.category,
        has_full_text: hit.body.is_some(),
    }
}

fn values<T>(pairs: Vec<(T, u64)>) -> Vec<FacetValue<T>> {
    pairs
        .into_iter()
        .map(|(value, count)| FacetValue { value, count })
        .collect()
}

fn facet_groups(facets: Facets) -> FacetGroups {
    FacetGroups {
        languages: values(facets.languages),
        sources: values(facets.sources),
        categories: values(facets.categories),
    }
}
