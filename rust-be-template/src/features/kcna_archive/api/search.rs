//! Search, histogram, and trends handlers.

use std::sync::Arc;

use axum::{Extension, extract::Query, response::IntoResponse};
use kcna_archive_core::service::trends::MAX_TREND_TERMS;

use super::{
    dto::{ArchiveHistogramResponse, ArchiveSearchResponse, ArchiveTrendsResponse},
    error_map::map_archive_error,
    query::{ArchiveQuery, TrendTermsQuery, trend_terms},
};
use crate::{
    dto::responses::response_data::http_resp,
    errors::code_error::{CodeErrorResp, HandlerResponse},
    features::kcna_archive::service::archive_service::KcnaArchiveService,
    util::time::now::tokio_now,
};

/// Searches the snapshot and returns one page with snippets and facet counts.
///
/// Answers 503 while the snapshot is not installed, loading, or failed; the
/// error code says which, and the status endpoint reports the same state as a
/// plain 200.
#[utoipa::path(
    get,
    path = "/api/admin/kcna-archive/search",
    tag = "kcna_archive",
    params(ArchiveQuery),
    responses(
        (status = 200, description = "One page of results", body = ArchiveSearchResponse),
        (status = 400, description = "Malformed filter, sort, or paging parameter", body = CodeErrorResp),
        (status = 401, body = CodeErrorResp), (status = 403, body = CodeErrorResp),
        (status = 500, body = CodeErrorResp),
        (status = 503, description = "No snapshot is available", body = CodeErrorResp)
    )
)]
pub async fn kcna_archive_search(
    Extension(service): Extension<Arc<KcnaArchiveService>>,
    Query(query): Query<ArchiveQuery>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let result = service
        .search(query.into())
        .await
        .map_err(map_archive_error)?;
    Ok(http_resp(ArchiveSearchResponse::from(result), (), start))
}

/// Counts matches per time bucket over the whole time axis.
#[utoipa::path(
    get,
    path = "/api/admin/kcna-archive/histogram",
    tag = "kcna_archive",
    params(ArchiveQuery),
    responses(
        (status = 200, description = "Matches per bucket", body = ArchiveHistogramResponse),
        (status = 400, description = "Malformed parameter or a day bucket over a long range", body = CodeErrorResp),
        (status = 401, body = CodeErrorResp), (status = 403, body = CodeErrorResp),
        (status = 500, body = CodeErrorResp),
        (status = 503, description = "No snapshot is available", body = CodeErrorResp)
    )
)]
pub async fn kcna_archive_histogram(
    Extension(service): Extension<Arc<KcnaArchiveService>>,
    Query(query): Query<ArchiveQuery>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let result = service
        .histogram(query.into())
        .await
        .map_err(map_archive_error)?;
    let response = ArchiveHistogramResponse::try_from(result).map_err(map_archive_error)?;
    Ok(http_resp(response, (), start))
}

/// Compares how many documents mention each of up to four terms per bucket.
///
/// The `term` parameter repeats, so the handler reads the raw pairs of the
/// query string instead of a typed struct.
#[utoipa::path(
    get,
    path = "/api/admin/kcna-archive/trends",
    tag = "kcna_archive",
    params(ArchiveQuery, TrendTermsQuery),
    responses(
        (status = 200, description = "Per-term counts parallel to the bucket starts", body = ArchiveTrendsResponse),
        (status = 400, description = "No term, more than four terms, or a malformed parameter", body = CodeErrorResp),
        (status = 401, body = CodeErrorResp), (status = 403, body = CodeErrorResp),
        (status = 500, body = CodeErrorResp),
        (status = 503, description = "No snapshot is available", body = CodeErrorResp)
    )
)]
pub async fn kcna_archive_trends(
    Extension(service): Extension<Arc<KcnaArchiveService>>,
    Query(query): Query<ArchiveQuery>,
    Query(pairs): Query<Vec<(String, String)>>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let terms = trend_terms(pairs, MAX_TREND_TERMS);
    let result = service
        .trends(query.into(), terms)
        .await
        .map_err(map_archive_error)?;
    let response = ArchiveTrendsResponse::try_from(result).map_err(map_archive_error)?;
    Ok(http_resp(response, (), start))
}
