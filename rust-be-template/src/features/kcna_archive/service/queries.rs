//! Read use cases; each runs one core call on the loaded snapshot.

use kcna_archive_core::service::{
    export::ExportFormat,
    models::{DocumentResponse, HistogramResponse, SearchResponse, TrendsResponse},
    params::QueryParams,
};

use super::archive_service::KcnaArchiveService;
use crate::features::kcna_archive::error::ArchiveError;

impl KcnaArchiveService {
    /// One page of results with snippets and facet counts.
    pub async fn search(&self, params: QueryParams) -> Result<SearchResponse, ArchiveError> {
        self.query(move |archive| archive.search(&params)).await
    }

    /// Matches per time bucket.
    pub async fn histogram(&self, params: QueryParams) -> Result<HistogramResponse, ArchiveError> {
        self.query(move |archive| archive.histogram(&params)).await
    }

    /// Document counts for up to four terms per time bucket.
    pub async fn trends(
        &self,
        params: QueryParams,
        terms: Vec<String>,
    ) -> Result<TrendsResponse, ArchiveError> {
        self.query(move |archive| archive.trends(&params, terms))
            .await
    }

    /// A result set serialized as CSV or JSON Lines.
    pub async fn export(
        &self,
        params: QueryParams,
        format: ExportFormat,
    ) -> Result<String, ArchiveError> {
        self.query(move |archive| archive.export(&params, format))
            .await
    }

    /// One document with the matches of `query` marked.
    pub async fn document(
        &self,
        id: String,
        query: String,
    ) -> Result<DocumentResponse, ArchiveError> {
        self.query(move |archive| archive.document(&id, &query))
            .await
    }
}
