//! Explicit conversion from the core result models to the API contracts.

use kcna_archive_core::{
    service::models::{
        DocumentResponse, FacetGroups, FacetValue, HistogramResponse, SearchItem, SearchResponse,
        StatusResponse, TextSegment, TrendsResponse,
    },
    types::{Language, PublicationPrecision, Source},
};
use tracing::error;

use super::{
    document_dto::{ArchiveCopyLink, ArchiveDocumentResponse, ArchiveTranslationLink},
    dto::{
        ArchiveBucket, ArchiveCategoryFacet, ArchiveFacets, ArchiveHistogramBucket,
        ArchiveHistogramResponse, ArchiveLanguage, ArchiveLanguageFacet, ArchivePrecision,
        ArchiveSearchItem, ArchiveSearchResponse, ArchiveSource, ArchiveSourceFacet,
        ArchiveTextSegment, ArchiveTrendSeries, ArchiveTrendsResponse,
    },
    status_dto::{ArchiveServiceState, ArchiveSnapshotStatus, ArchiveStatusResponse},
};
use crate::features::kcna_archive::{
    error::ArchiveError,
    service::state::{ArchiveStatus, ServiceState},
};

impl From<Language> for ArchiveLanguage {
    fn from(language: Language) -> Self {
        match language {
            Language::Korean => Self::Kor,
            Language::English => Self::Eng,
            Language::Chinese => Self::Zho,
            Language::Russian => Self::Rus,
            Language::Spanish => Self::Spa,
            Language::Japanese => Self::Jpn,
        }
    }
}

impl From<Source> for ArchiveSource {
    fn from(source: Source) -> Self {
        match source {
            Source::KcnaKp => Self::KcnaKp,
            Source::KcnaCoJp => Self::KcnaCoJp,
            Source::StilesArchive => Self::StilesArchive,
        }
    }
}

impl From<PublicationPrecision> for ArchivePrecision {
    fn from(precision: PublicationPrecision) -> Self {
        match precision {
            PublicationPrecision::Second => Self::Second,
            PublicationPrecision::Minute => Self::Minute,
            PublicationPrecision::Day => Self::Day,
            PublicationPrecision::InferredDay => Self::InferredDay,
        }
    }
}

/// The core reports bucket sizes by name; an unknown name is a contract break
/// between the crates, so it is logged and reported as an internal failure.
fn bucket(name: &str) -> Result<ArchiveBucket, ArchiveError> {
    match name {
        "year" => Ok(ArchiveBucket::Year),
        "month" => Ok(ArchiveBucket::Month),
        "day" => Ok(ArchiveBucket::Day),
        other => {
            error!(
                bucket = other,
                "KCNA archive returned an unknown bucket size"
            );
            Err(ArchiveError::Internal)
        }
    }
}

impl From<TextSegment> for ArchiveTextSegment {
    fn from(segment: TextSegment) -> Self {
        Self {
            text: segment.text,
            hit: segment.hit,
        }
    }
}

fn segments(values: Vec<TextSegment>) -> Vec<ArchiveTextSegment> {
    values.into_iter().map(ArchiveTextSegment::from).collect()
}

impl From<SearchItem> for ArchiveSearchItem {
    fn from(item: SearchItem) -> Self {
        Self {
            id: item.id,
            title: segments(item.title),
            snippet: segments(item.snippet),
            publication_date: item.publication_date,
            language: item.language.into(),
            sources: item.sources.into_iter().map(Into::into).collect(),
            category: item.category,
            has_full_text: item.has_full_text,
        }
    }
}

fn language_facets(values: Vec<FacetValue<Language>>) -> Vec<ArchiveLanguageFacet> {
    values
        .into_iter()
        .map(|facet| ArchiveLanguageFacet {
            value: facet.value.into(),
            count: facet.count,
        })
        .collect()
}

fn source_facets(values: Vec<FacetValue<Source>>) -> Vec<ArchiveSourceFacet> {
    values
        .into_iter()
        .map(|facet| ArchiveSourceFacet {
            value: facet.value.into(),
            count: facet.count,
        })
        .collect()
}

impl From<FacetGroups> for ArchiveFacets {
    fn from(facets: FacetGroups) -> Self {
        Self {
            languages: language_facets(facets.languages),
            sources: source_facets(facets.sources),
            categories: facets
                .categories
                .into_iter()
                .map(|facet| ArchiveCategoryFacet {
                    value: facet.value,
                    count: facet.count,
                })
                .collect(),
        }
    }
}

impl From<SearchResponse> for ArchiveSearchResponse {
    fn from(response: SearchResponse) -> Self {
        Self {
            total: response.total,
            offset: response.offset,
            limit: response.limit,
            items: response.items.into_iter().map(Into::into).collect(),
            facets: response.facets.into(),
        }
    }
}

impl TryFrom<HistogramResponse> for ArchiveHistogramResponse {
    type Error = ArchiveError;

    fn try_from(response: HistogramResponse) -> Result<Self, ArchiveError> {
        Ok(Self {
            bucket: bucket(response.bucket)?,
            buckets: response
                .buckets
                .into_iter()
                .map(|bucket| ArchiveHistogramBucket {
                    start: bucket.start,
                    count: bucket.count,
                })
                .collect(),
        })
    }
}

impl TryFrom<TrendsResponse> for ArchiveTrendsResponse {
    type Error = ArchiveError;

    fn try_from(response: TrendsResponse) -> Result<Self, ArchiveError> {
        Ok(Self {
            bucket: bucket(response.bucket)?,
            starts: response.starts,
            totals: response.totals,
            series: response
                .series
                .into_iter()
                .map(|series| ArchiveTrendSeries {
                    term: series.term,
                    counts: series.counts,
                })
                .collect(),
        })
    }
}

impl From<DocumentResponse> for ArchiveDocumentResponse {
    fn from(document: DocumentResponse) -> Self {
        Self {
            id: document.id,
            title: segments(document.title),
            paragraphs: document.paragraphs.into_iter().map(segments).collect(),
            publication_date: document.publication_date,
            publication_precision: document.publication_precision.into(),
            language: document.language.into(),
            source: document.source.into(),
            sources: document.sources.into_iter().map(Into::into).collect(),
            canonical_url: document.canonical_url,
            category: document.category,
            source_record_id: document.source_record_id,
            metadata: document.metadata,
            translations: document
                .translations
                .into_iter()
                .map(|link| ArchiveTranslationLink {
                    id: link.id,
                    language: link.language.into(),
                    title: link.title,
                })
                .collect(),
            copies: document
                .copies
                .into_iter()
                .map(|link| ArchiveCopyLink {
                    id: link.id,
                    source: link.source.into(),
                    canonical_url: link.canonical_url,
                })
                .collect(),
        }
    }
}

impl From<ServiceState> for ArchiveServiceState {
    fn from(state: ServiceState) -> Self {
        match state {
            ServiceState::NotInstalled => Self::NotInstalled,
            ServiceState::Loading => Self::Loading,
            ServiceState::Ready => Self::Ready,
            ServiceState::Failed => Self::Failed,
        }
    }
}

impl From<StatusResponse> for ArchiveSnapshotStatus {
    fn from(status: StatusResponse) -> Self {
        Self {
            documents: status.documents,
            copies: status.copies,
            indexed: status.indexed,
            translation_groups: status.translation_groups,
            oldest_date: status.oldest_date,
            newest_date: status.newest_date,
            last_retrieved_at: status.last_retrieved_at,
            snapshot_built_at: status.snapshot_built_at,
            memory_bytes: status.memory_bytes,
            languages: language_facets(status.languages),
            sources: source_facets(status.sources),
        }
    }
}

impl From<ArchiveStatus> for ArchiveStatusResponse {
    fn from(status: ArchiveStatus) -> Self {
        Self {
            state: status.state.into(),
            reloading: status.reloading,
            last_reload_failed: status.last_reload_failed,
            snapshot: status.snapshot.map(Into::into),
        }
    }
}
