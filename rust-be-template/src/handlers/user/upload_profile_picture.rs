//! Streamed profile-picture replacement with object-store compensation.

use std::sync::Arc;

use axum::{
    Extension,
    extract::{Multipart, State},
};
use tracing::{error, warn};
use uuid::Uuid;

use crate::{
    dto::responses::response_data::{Response as ApiResponse, http_resp},
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    features::accounts::error::AccountError,
    init::state::ServerState,
    util::{
        image::{
            image_variant::{CyhdevImageType, IMAGE_ENCODING_FORMAT},
            map_image_format_to_db_enum::map_image_format_to_str,
            process_uploaded_image_files::process_uploaded_image_files,
        },
        media::{
            cleanup::settle_durable_cleanup,
            image_upload::{has_file_extension, is_allowed_image_mime},
            object_store::{ObjectLocation, S3MediaObjectStore},
            persistence::{
                CleanupFailure, MediaWriteError, PendingMediaObject, PersistedMedia,
                persist_media_objects,
            },
            staged_upload::{StageUploadError, StagedUpload, stage_file_field},
        },
        s3::AWS_S3_BUCKET_NAME,
        time::now::tokio_now,
    },
};

const MAX_PROFILE_PICTURE_BYTES: u64 = 10 * 1024 * 1024;

#[utoipa::path(
    post,
    path = "/api/user/upload-profile-picture",
    tag = "user",
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
    let mut processed =
        process_uploaded_image_files(upload.path(), None, vec![CyhdevImageType::ProfilePicture])
            .await
            .map_err(|source| {
                error!(error = %source, user_id = %user_id, "Failed to process profile picture");
                code_err(CodeError::COULD_NOT_PROCESS_IMAGE, source)
            })?
            .into_iter();
    let processed = processed.next().ok_or_else(|| {
        code_err(
            CodeError::COULD_NOT_PROCESS_IMAGE,
            "Profile-picture encoder produced no output",
        )
    })?;
    drop(upload);

    let image_id = Uuid::now_v7();
    let (extension, image_type) = map_image_format_to_str(IMAGE_ENCODING_FORMAT);
    let location =
        ObjectLocation::new(AWS_S3_BUCKET_NAME, format!("images/{image_id}.{extension}"));
    let region = state
        .aws_profile_picture_config
        .region()
        .map(|region| region.to_string())
        .unwrap_or_else(|| "us-west-1".to_string());
    let object_url = location.public_s3_url(&region);
    let pending = [PendingMediaObject {
        location: location.clone(),
        content_type: "image/avif".to_string(),
        source: processed.path_buf(),
    }];
    let store = S3MediaObjectStore::from_config(&state.aws_profile_picture_config);
    let account_service = state.account_service();
    let persistence_account_service = Arc::clone(&account_service);

    let result = persist_media_objects(&store, &pending, async move {
        let replacement = persistence_account_service
            .replace_profile_picture_metadata(user_id, image_type, &object_url)
            .await?;
        let superseded = replacement
            .cleanup_objects
            .iter()
            .map(|cleanup| cleanup.location.clone())
            .collect();
        Ok::<_, AccountError>(PersistedMedia::new(replacement, superseded))
    })
    .await;

    match result {
        Ok(success) => {
            if success.value.unresolved_cleanup_count > 0 {
                warn!(
                    user_id = %user_id,
                    unresolved = success.value.unresolved_cleanup_count,
                    "Superseded legacy profile media requires administrative resolution"
                );
            }
            settle_durable_cleanup(
                &account_service,
                success.value.cleanup_objects,
                &success.cleaned,
                &success.cleanup_failures,
            )
            .await;
            log_cleanup_failures(user_id, &success.cleanup_failures);
        }
        Err(MediaWriteError::Upload {
            source,
            compensation_failures,
        }) => {
            log_cleanup_failures(user_id, &compensation_failures);
            error!(
                user_id = %user_id,
                key = %location.key(),
                operation = ?source.operation(),
                retryable = source.is_retryable(),
                error = %source,
                "Failed to upload profile picture"
            );
            return Err(code_err(CodeError::FILE_UPLOAD_ERROR, source));
        }
        Err(MediaWriteError::Persistence {
            source,
            compensation_failures,
        }) => {
            log_cleanup_failures(user_id, &compensation_failures);
            let code = match &source {
                AccountError::Pool(_) => CodeError::POOL_ERROR,
                _ => CodeError::DB_INSERTION_ERROR,
            };
            error!(error = %source, user_id = %user_id, "Failed to commit profile-picture metadata");
            return Err(code_err(code, source));
        }
    }

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
        return Err(code_err(
            CodeError::FILE_UPLOAD_ERROR,
            "Unsupported image type",
        ));
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

fn log_cleanup_failures(user_id: Uuid, failures: &[CleanupFailure]) {
    for failure in failures {
        error!(
            user_id = %user_id,
            bucket = %failure.location.bucket(),
            key = %failure.location.key(),
            retryable = failure.is_retryable(),
            error = %failure.error,
            "Media cleanup remains pending"
        );
    }
}
