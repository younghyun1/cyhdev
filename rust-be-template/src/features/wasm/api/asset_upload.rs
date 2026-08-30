//! Bounded multipart parsing for WebAssembly assets.

use axum::{
    extract::{Multipart, multipart::MultipartError},
    http::StatusCode,
};
use tracing::{error, info};

use crate::{
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    util::media::{
        image_upload::is_allowed_image_mime,
        staged_upload::{StageUploadError, read_bounded_text_field, stage_file_field},
    },
};

use super::super::service::{
    asset_inputs::{StagedBundleUpload, StagedWasmAssets},
    bundle_processing::MAX_BUNDLE_SIZE_BYTES,
};

const MAX_THUMBNAIL_SIZE_BYTES: u64 = 10 * 1024 * 1024;
const MAX_TITLE_BYTES: usize = 512;
const MAX_DESCRIPTION_BYTES: usize = 64 * 1024;

pub async fn read_assets(multipart: &mut Multipart) -> HandlerResponse<StagedWasmAssets> {
    let mut result = StagedWasmAssets {
        bundle: None,
        thumbnail: None,
        title: None,
        description: None,
    };
    while let Some(field) = multipart.next_field().await.map_err(map_multipart_error)? {
        let name = field.name().map(str::to_owned);
        match name.as_deref() {
            Some("bundle_file") | Some("wasm_file") | Some("wasm") => {
                if result.bundle.is_some() {
                    return Err(duplicate_field("bundle"));
                }
                let source = stage_file_field(field, MAX_BUNDLE_SIZE_BYTES)
                    .await
                    .map_err(map_stage_error)?;
                result.bundle = Some(StagedBundleUpload { source });
            }
            Some("thumbnail") | Some("thumbnail_file") => {
                if result.thumbnail.is_some() {
                    return Err(duplicate_field("thumbnail"));
                }
                if let Some(content_type) = field.content_type()
                    && !is_allowed_image_mime(content_type)
                {
                    return Err(code_err(
                        CodeError::INVALID_REQUEST,
                        "Unsupported thumbnail image type",
                    ));
                }
                result.thumbnail = Some(
                    stage_file_field(field, MAX_THUMBNAIL_SIZE_BYTES)
                        .await
                        .map_err(map_stage_error)?,
                );
            }
            Some("title") | Some("wasm_module_title") => {
                if result.title.is_some() {
                    return Err(duplicate_field("title"));
                }
                result.title = Some(
                    read_bounded_text_field(field, MAX_TITLE_BYTES)
                        .await
                        .map_err(map_stage_error)?,
                );
            }
            Some("description") | Some("wasm_module_description") => {
                if result.description.is_some() {
                    return Err(duplicate_field("description"));
                }
                result.description = Some(
                    read_bounded_text_field(field, MAX_DESCRIPTION_BYTES)
                        .await
                        .map_err(map_stage_error)?,
                );
            }
            Some(other) => info!(field = other, "Ignored unknown WebAssembly asset field"),
            None => info!("Ignored unnamed WebAssembly asset field"),
        }
    }
    Ok(result)
}

fn duplicate_field(name: &'static str) -> CodeErrorResp {
    code_err(
        CodeError::INVALID_REQUEST,
        format!("Only one {name} field is allowed"),
    )
}

fn map_stage_error(error: StageUploadError) -> CodeErrorResp {
    let code = match &error {
        StageUploadError::Multipart(source) => multipart_code(source.status()),
        StageUploadError::TooLarge { .. } => CodeError::UPLOAD_TOO_LARGE,
        StageUploadError::Empty | StageUploadError::InvalidUtf8(_) => CodeError::INVALID_REQUEST,
        StageUploadError::Io(_) => CodeError::FILE_UPLOAD_ERROR,
    };
    code_err(code, error)
}

fn map_multipart_error(error: MultipartError) -> CodeErrorResp {
    error!(status = %error.status(), error = %error, "Failed to read WebAssembly multipart field");
    code_err(multipart_code(error.status()), error)
}

fn multipart_code(status: StatusCode) -> CodeError {
    match status {
        StatusCode::PAYLOAD_TOO_LARGE => CodeError::UPLOAD_TOO_LARGE,
        status if status.is_client_error() => CodeError::INVALID_REQUEST,
        _ => CodeError::FILE_UPLOAD_ERROR,
    }
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;

    use super::{duplicate_field, map_stage_error, multipart_code};
    use crate::{errors::code_error::CodeError, util::media::staged_upload::StageUploadError};

    #[test]
    fn multipart_statuses_preserve_client_and_server_ownership() {
        assert_eq!(
            multipart_code(StatusCode::BAD_REQUEST).error_code,
            CodeError::INVALID_REQUEST.error_code
        );
        assert_eq!(
            multipart_code(StatusCode::PAYLOAD_TOO_LARGE).error_code,
            CodeError::UPLOAD_TOO_LARGE.error_code
        );
        assert_eq!(
            multipart_code(StatusCode::INTERNAL_SERVER_ERROR).error_code,
            CodeError::FILE_UPLOAD_ERROR.error_code
        );
    }

    #[test]
    fn bounded_stage_failures_do_not_become_server_errors() {
        let oversized = map_stage_error(StageUploadError::TooLarge { limit_bytes: 1 });
        let empty = map_stage_error(StageUploadError::Empty);
        assert_eq!(oversized.http_status_code, StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(empty.http_status_code, StatusCode::BAD_REQUEST);
        assert_eq!(
            duplicate_field("bundle").http_status_code,
            StatusCode::BAD_REQUEST
        );
    }
}
