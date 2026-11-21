//! Single-document handler.

use std::sync::Arc;

use axum::{
    Extension,
    extract::{Path, Query},
    response::IntoResponse,
};

use super::{
    document_dto::ArchiveDocumentResponse, error_map::map_archive_error, query::DocumentQuery,
};
use crate::{
    dto::responses::response_data::http_resp,
    errors::code_error::{CodeErrorResp, HandlerResponse},
    features::kcna_archive::service::archive_service::KcnaArchiveService,
    util::time::now::tokio_now,
};

/// Returns one document, with the matches of `q` marked in its title and body.
#[utoipa::path(
    get,
    path = "/api/admin/kcna-archive/documents/{document_id}",
    tag = "kcna_archive",
    params(
        ("document_id" = String, Path, description = "Document UUID"),
        DocumentQuery
    ),
    responses(
        (status = 200, description = "The document", body = ArchiveDocumentResponse),
        (status = 400, description = "The id is not a UUID", body = CodeErrorResp),
        (status = 401, body = CodeErrorResp),
        (status = 403, body = CodeErrorResp),
        (status = 404, description = "No such document", body = CodeErrorResp),
        (status = 500, body = CodeErrorResp),
        (status = 503, description = "No snapshot is available", body = CodeErrorResp)
    )
)]
pub async fn kcna_archive_document(
    Extension(service): Extension<Arc<KcnaArchiveService>>,
    Path(document_id): Path<String>,
    Query(query): Query<DocumentQuery>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let result = service
        .document(document_id, query.q.unwrap_or_default())
        .await
        .map_err(map_archive_error)?;
    Ok(http_resp(ArchiveDocumentResponse::from(result), (), start))
}
