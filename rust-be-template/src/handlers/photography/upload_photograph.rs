//! Streamed single-photograph upload and compensated persistence.

use std::sync::Arc;

use axum::{
    Extension,
    extract::{Multipart, State},
};
use diesel_async::RunQueryDsl;
use tracing::{error, warn};
use uuid::Uuid;

use crate::{
    domain::photography::photographs::{Photograph, PhotographInsertable},
    dto::responses::response_data::{Response as ApiResponse, http_resp},
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    handlers::photography::photograph_upload_request::PhotographUploadRequest,
    init::state::ServerState,
    schema::photographs,
    util::{
        image::{
            exif_utils::extract_exif_shot_at_from_path,
            map_image_format_to_db_enum::map_image_format_to_str,
            process_uploaded_image_files::process_uploaded_image_files,
            image_variant::{CyhdevImageType, IMAGE_ENCODING_FORMAT},
        },
        media::{
            object_store::{ObjectLocation, S3MediaObjectStore},
            persistence::{
                CleanupFailure, MediaWriteError, PendingMediaObject, PersistedMedia,
                persist_media_objects,
            },
        },
        s3::AWS_S3_BUCKET_NAME,
        time::now::tokio_now,
    },
};

#[derive(Debug, thiserror::Error)]
enum PhotographPersistenceError {
    #[error("database pool checkout failed: {0}")]
    Pool(#[source] anyhow::Error),
    #[error("photograph insert failed: {0}")]
    Insert(#[source] diesel::result::Error),
}

#[utoipa::path(
    post,
    path = "/api/photographs/upload",
    tag = "photography",
    request_body(content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Photograph uploaded successfully", body = Photograph),
        (status = 400, description = "Invalid upload payload", body = CodeErrorResp),
        (status = 401, description = "Unauthorized", body = CodeErrorResp),
        (status = 403, description = "Forbidden (not superuser)", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn upload_photograph(
    Extension(user_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    mut multipart: Multipart,
) -> HandlerResponse<ApiResponse<Photograph, ()>> {
    let start = tokio_now();
    let upload = PhotographUploadRequest::read(&mut multipart, user_id).await?;
    let source_path = upload.source.path().to_path_buf();
    let photograph_shot_at = read_exif(source_path.clone(), user_id).await;
    let mut outputs = process_uploaded_image_files(
        &source_path,
        None,
        vec![CyhdevImageType::Photograph, CyhdevImageType::Thumbnail],
    )
    .await
    .map_err(|source| {
        error!(error = %source, user_id = %user_id, "Failed to process photograph");
        code_err(CodeError::COULD_NOT_PROCESS_IMAGE, source)
    })?
    .into_iter();
    let main = outputs.next().ok_or_else(|| {
        code_err(
            CodeError::COULD_NOT_PROCESS_IMAGE,
            "Photograph encoder produced no main image",
        )
    })?;
    let thumbnail = outputs.next().ok_or_else(|| {
        code_err(
            CodeError::COULD_NOT_PROCESS_IMAGE,
            "Photograph encoder produced no thumbnail",
        )
    })?;
    drop(source_path);
    let PhotographUploadRequest {
        source,
        comments,
        latitude,
        longitude,
        context,
    } = upload;
    drop(source);

    let image_id = Uuid::now_v7();
    let (extension, image_type) = map_image_format_to_str(IMAGE_ENCODING_FORMAT);
    let image_location =
        ObjectLocation::new(AWS_S3_BUCKET_NAME, format!("images/{image_id}.{extension}"));
    let thumbnail_location = ObjectLocation::new(
        AWS_S3_BUCKET_NAME,
        format!("thumbnails/{image_id}.{extension}"),
    );
    let region = state
        .aws_profile_picture_config
        .region()
        .map(|region| region.to_string())
        .unwrap_or_else(|| "us-west-1".to_string());
    let object_url = image_location.public_s3_url(&region);
    let thumbnail_url = thumbnail_location.public_s3_url(&region);
    let pending = [
        PendingMediaObject {
            location: image_location,
            content_type: "image/avif".to_string(),
            source: main.path_buf(),
        },
        PendingMediaObject {
            location: thumbnail_location,
            content_type: "image/avif".to_string(),
            source: thumbnail.path_buf(),
        },
    ];
    let store = S3MediaObjectStore::from_config(&state.aws_profile_picture_config);

    let result = persist_media_objects(&store, &pending, async move {
        let mut connection = state
            .get_conn()
            .await
            .map_err(PhotographPersistenceError::Pool)?;
        let insert = PhotographInsertable {
            user_id,
            photograph_shot_at,
            photograph_image_type: image_type,
            photograph_context: context,
            photograph_is_on_cloud: true,
            photograph_link: object_url.clone(),
            photograph_comments: comments,
            photograph_lat: latitude,
            photograph_lon: longitude,
            photograph_thumbnail_link: thumbnail_url.clone(),
        };
        let photograph = diesel::insert_into(photographs::table)
            .values(insert)
            .get_result(&mut connection)
            .await
            .map_err(PhotographPersistenceError::Insert)?;
        drop(connection);
        Ok(PersistedMedia::new(photograph, Vec::new()))
    })
    .await;

    match result {
        Ok(success) => {
            log_cleanup_failures(user_id, &success.cleanup_failures);
            Ok(http_resp(success.value, (), start))
        }
        Err(MediaWriteError::Upload {
            source,
            compensation_failures,
        }) => {
            log_cleanup_failures(user_id, &compensation_failures);
            error!(error = %source, user_id = %user_id, retryable = source.is_retryable(), "Failed to upload photograph objects");
            Err(code_err(CodeError::FILE_UPLOAD_ERROR, source))
        }
        Err(MediaWriteError::Persistence {
            source,
            compensation_failures,
        }) => {
            log_cleanup_failures(user_id, &compensation_failures);
            let code = match &source {
                PhotographPersistenceError::Pool(_) => CodeError::POOL_ERROR,
                PhotographPersistenceError::Insert(_) => CodeError::DB_INSERTION_ERROR,
            };
            error!(error = %source, user_id = %user_id, "Failed to commit photograph metadata");
            Err(code_err(code, source))
        }
    }
}

async fn read_exif(
    path: std::path::PathBuf,
    user_id: Uuid,
) -> Option<chrono::DateTime<chrono::Utc>> {
    match tokio::task::spawn_blocking(move || extract_exif_shot_at_from_path(&path)).await {
        Ok(Ok(value)) => value,
        Ok(Err(source)) => {
            warn!(error = %source, user_id = %user_id, "Failed to parse photograph EXIF");
            None
        }
        Err(source) => {
            error!(error = %source, user_id = %user_id, "EXIF worker failed");
            None
        }
    }
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
