//! Authenticated bounded multipart adapter for profile-picture uploads.

use std::sync::Arc;

use axum::{Extension, extract::{Multipart, State}};
use tracing::{error, warn};
use uuid::Uuid;

use crate::{
    dto::responses::response_data::{Response as ApiResponse, http_resp},
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    features::accounts::{
        error::AccountError,
        service::profile_picture_upload::ProfilePictureUploadError,
    },
    init::state::ServerState,
    util::{
        media::{
            image_upload::{has_file_extension, is_allowed_image_mime},
            staged_upload::{StageUploadError, StagedUpload, stage_file_field},
        },
        time::now::tokio_now,
    },
};

const MAX_PROFILE_PICTURE_BYTES: u64 = 10 * 1024 * 1024;

#[utoipa::path(post, path = "/api/user/upload-profile-picture", tag = "user",
    request_body(content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Profile picture uploaded successfully"),
        (status = 400, description = "Invalid upload payload", body = CodeErrorResp),
        (status = 401, description = "Unauthorized", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn upload_profile_picture(
    Extension(user_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    mut multipart: Multipart,
) -> HandlerResponse<ApiResponse<(), ()>> {
    let start = tokio_now();
    let upload = read_profile_picture(&mut multipart, user_id).await?;
    state
        .account_service()
        .upload_profile_picture(user_id, upload)
        .await
        .map_err(map_upload_error)?;
    Ok(http_resp((), (), start))
}

async fn read_profile_picture(
    multipart: &mut Multipart,
    user_id: Uuid,
) -> HandlerResponse<StagedUpload> {
    let mut upload = None;
    while let Some(field) = multipart.next_field().await.map_err(|source| {
        error!(error = %source, user_id = %user_id, "Failed to read multipart field");
        code_err(CodeError::FILE_UPLOAD_ERROR, source)
    })? {
        if upload.is_some() {
            return Err(code_err(
                CodeError::FILE_UPLOAD_ERROR,
                "Only one profile picture may be uploaded",
            ));
        }
        validate_file_metadata(field.file_name(), field.content_type(), user_id)?;
        upload = Some(
            stage_file_field(field, MAX_PROFILE_PICTURE_BYTES)
                .await
                .map_err(map_stage_error)?,
        );
    }
    upload.ok_or_else(|| code_err(CodeError::FILE_UPLOAD_ERROR, "File is empty"))
}

fn validate_file_metadata(
    file_name: Option<&str>,
    content_type: Option<&str>,
    user_id: Uuid,
) -> HandlerResponse<()> {
    let file_name = file_name.ok_or_else(|| {
        warn!(user_id = %user_id, "Profile picture is missing a filename");
        code_err(CodeError::FILE_UPLOAD_ERROR, "Filename is required")
    })?;
    if !has_file_extension(file_name) {
        return Err(code_err(
            CodeError::FILE_UPLOAD_ERROR,
            "Filename extension is required",
        ));
    }
    let content_type = content_type
        .ok_or_else(|| code_err(CodeError::FILE_UPLOAD_ERROR, "Content type is required"))?;
    if !is_allowed_image_mime(content_type) {
        return Err(code_err(CodeError::FILE_UPLOAD_ERROR, "Unsupported image type"));
    }
    Ok(())
}

fn map_stage_error(source: StageUploadError) -> CodeErrorResp {
    let code = if matches!(&source, StageUploadError::TooLarge { .. }) {
        CodeError::IMAGE_TOO_LARGE
    } else {
        CodeError::FILE_UPLOAD_ERROR
    };
    code_err(code, source)
}

fn map_upload_error(error: ProfilePictureUploadError) -> CodeErrorResp {
    match error {
        ProfilePictureUploadError::Processing(source) => {
            code_err(CodeError::COULD_NOT_PROCESS_IMAGE, source)
        }
        ProfilePictureUploadError::Upload(source) => {
            code_err(CodeError::FILE_UPLOAD_ERROR, source)
        }
        ProfilePictureUploadError::Persistence(source) => {
            let code = match &source {
                AccountError::Pool(_) => CodeError::POOL_ERROR,
                _ => CodeError::DB_INSERTION_ERROR,
            };
            code_err(code, source)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_PROFILE_PICTURE_BYTES, map_stage_error, validate_file_metadata};
    use crate::{errors::code_error::CodeError, util::media::staged_upload::StageUploadError};
    use uuid::Uuid;

    #[test]
    fn metadata_requires_one_supported_named_image() {
        let user_id = Uuid::nil();
        assert!(validate_file_metadata(Some("avatar.jpg"), Some("image/jpeg"), user_id).is_ok());
        assert!(validate_file_metadata(None, Some("image/jpeg"), user_id).is_err());
        assert!(validate_file_metadata(Some("avatar"), Some("image/jpeg"), user_id).is_err());
        assert!(validate_file_metadata(Some("avatar.jpg"), None, user_id).is_err());
        assert!(validate_file_metadata(Some("avatar.txt"), Some("text/plain"), user_id).is_err());
    }

    #[test]
    fn oversized_staging_uses_the_image_size_error() {
        let response = map_stage_error(StageUploadError::TooLarge {
            limit_bytes: MAX_PROFILE_PICTURE_BYTES,
        });
        assert_eq!(response.error_code, CodeError::IMAGE_TOO_LARGE.error_code);
    }
}
