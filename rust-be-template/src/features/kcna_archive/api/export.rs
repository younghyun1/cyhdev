//! Export handler: a file download rather than a JSON envelope.

use std::sync::Arc;

use axum::{
    Extension,
    extract::Query,
    http::{HeaderValue, header},
    response::{IntoResponse, Response},
};
use kcna_archive_core::service::export::ExportFormat;

use super::{
    error_map::map_archive_error,
    query::{ArchiveQuery, ExportQuery},
};
use crate::{
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    features::kcna_archive::service::archive_service::KcnaArchiveService,
};

/// Downloads up to 10000 results as CSV or JSON Lines.
///
/// This is a deliberate exception to the JSON envelope, like the other file
/// and binary responses: the body is the file, sent with
/// `Content-Disposition: attachment`. Errors keep the normal JSON envelope.
/// The admin response policy adds `Cache-Control: no-store` because the text
/// is not redistributable.
#[utoipa::path(
    get,
    path = "/api/kcna-archive/export",
    tag = "kcna_archive",
    params(ArchiveQuery, ExportQuery),
    responses(
        (status = 200, description = "The result set as a file attachment", content(
            (String = "text/csv"),
            (String = "application/x-ndjson")
        )),
        (status = 400, description = "Unknown format or malformed parameter", body = CodeErrorResp),
        (status = 401, body = CodeErrorResp),
        (status = 403, body = CodeErrorResp),
        (status = 500, body = CodeErrorResp),
        (status = 503, description = "No snapshot is available", body = CodeErrorResp)
    )
)]
pub async fn kcna_archive_export(
    Extension(service): Extension<Arc<KcnaArchiveService>>,
    Query(query): Query<ArchiveQuery>,
    Query(export): Query<ExportQuery>,
) -> HandlerResponse<impl IntoResponse> {
    let Some(format) = ExportFormat::parse(export.format.as_deref()) else {
        return Err(map_archive_error(
            crate::features::kcna_archive::error::ArchiveError::BadRequest,
        ));
    };
    let body = service
        .export(query.into(), format)
        .await
        .map_err(map_archive_error)?;
    download(format, body)
}

/// Builds the attachment response for a serialized result set.
fn download(format: ExportFormat, body: String) -> Result<Response, CodeErrorResp> {
    let content_type = HeaderValue::from_str(format.media_type());
    let disposition = HeaderValue::from_str(&format!(
        "attachment; filename=\"kcna-archive-export.{}\"",
        format.extension()
    ));
    let (Ok(content_type), Ok(disposition)) = (content_type, disposition) else {
        return Err(code_err(
            CodeError::KCNA_ARCHIVE_INTERNAL,
            "export headers could not be encoded",
        ));
    };
    let mut response = body.into_response();
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, content_type);
    headers.insert(header::CONTENT_DISPOSITION, disposition);
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_and_jsonl_downloads_carry_their_type_and_filename() -> Result<(), CodeErrorResp> {
        for (format, media, name) in [
            (
                ExportFormat::Csv,
                "text/csv; charset=utf-8",
                "kcna-archive-export.csv",
            ),
            (
                ExportFormat::JsonLines,
                "application/x-ndjson",
                "kcna-archive-export.jsonl",
            ),
        ] {
            let response = download(format, "row\n".to_owned())?;
            assert_eq!(response.headers()[header::CONTENT_TYPE], media);
            let disposition = response.headers()[header::CONTENT_DISPOSITION]
                .to_str()
                .map(str::to_owned)
                .unwrap_or_default();
            assert_eq!(disposition, format!("attachment; filename=\"{name}\""));
            assert_eq!(
                response.headers()[header::X_CONTENT_TYPE_OPTIONS],
                "nosniff"
            );
        }
        Ok(())
    }
}
