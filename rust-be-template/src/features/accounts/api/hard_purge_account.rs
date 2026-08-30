//! Separately authorized hard purge of retained account identity.

use std::sync::Arc;

use axum::{
    Extension,
    extract::{Path, State},
    response::IntoResponse,
};
use futures_util::{StreamExt, stream};
use tracing::warn;
use uuid::Uuid;

use crate::{
    dto::responses::{
        auth::hard_purge_account_response::{
            HardPurgeAccountResponse, ProfileObjectCleanupFailure,
        },
        response_data::http_resp,
    },
    errors::code_error::{CodeErrorResp, HandlerResponse},
    features::accounts::{
        api::account_error::{AccountMutation, map_account_error},
        domain::lifecycle::ProfileObjectCleanup,
    },
    init::state::ServerState,
    util::{
        media::object_store::{MediaObjectStore, ObjectLocation, S3MediaObjectStore},
        s3::AWS_S3_BUCKET_NAME,
        time::now::tokio_now,
    },
};

const PROFILE_OBJECT_DELETE_CONCURRENCY: usize = 8;

#[utoipa::path(
    post,
    path = "/api/admin/users/{user_id}/hard-purge",
    tag = "admin",
    params(("user_id" = Uuid, Path, description = "Deleted account identifier")),
    responses(
        (status = 200, description = "Retained identity purged with explicit profile cleanup status", body = HardPurgeAccountResponse),
        (status = 401, description = "Authentication required", body = CodeErrorResp),
        (status = 403, description = "Superuser role required or system actor protected", body = CodeErrorResp),
        (status = 404, description = "Account not found", body = CodeErrorResp),
        (status = 409, description = "Account lifecycle conflict", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn hard_purge_account(
    Extension(requester_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    Path(user_id): Path<Uuid>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let receipt = state
        .account_service()
        .hard_purge_account(requester_id, user_id)
        .await
        .map_err(|error| map_account_error(error, AccountMutation::Update))?;
    let store = S3MediaObjectStore::from_config(&state.aws_profile_picture_config);
    let cleanup_results = stream::iter(
        receipt
            .profile_objects
            .into_iter()
            .map(|profile_object| delete_profile_object(&store, user_id, profile_object)),
    )
    .buffer_unordered(PROFILE_OBJECT_DELETE_CONCURRENCY)
    .collect::<Vec<_>>()
    .await;
    let mut deleted_profile_ids = Vec::with_capacity(cleanup_results.len());
    let mut failures = Vec::new();
    for result in cleanup_results {
        match result {
            Ok(profile_picture_id) => deleted_profile_ids.push(profile_picture_id),
            Err(failure) => failures.push(failure),
        }
    }
    let remotely_deleted = deleted_profile_ids.len();
    let finalized_cloud_metadata = state
        .account_service()
        .finalize_profile_cleanup(requester_id, user_id, &deleted_profile_ids)
        .await
        .map_err(|error| map_account_error(error, AccountMutation::Update))?;
    let profile_metadata_deleted =
        receipt.profile_metadata_deleted + finalized_cloud_metadata.metadata_deleted;
    let profile_cleanup_remaining = finalized_cloud_metadata.metadata_remaining;

    Ok(http_resp(
        HardPurgeAccountResponse {
            user_id: receipt.user_id,
            hard_purged_at: receipt.hard_purged_at,
            profile_objects_deleted: remotely_deleted,
            profile_metadata_deleted,
            profile_cleanup_remaining,
            profile_cleanup_failures: failures,
        },
        (),
        start,
    ))
}

async fn delete_profile_object(
    store: &S3MediaObjectStore,
    user_id: Uuid,
    profile_object: ProfileObjectCleanup,
) -> Result<Uuid, ProfileObjectCleanupFailure> {
    let object_url = profile_object.object_url.ok_or_else(|| ProfileObjectCleanupFailure {
        profile_picture_id: profile_object.profile_picture_id,
        object_url: None,
        reason: "profile metadata has no object URL".to_string(),
        retryable: false,
    })?;
    let location = ObjectLocation::from_public_s3_url(AWS_S3_BUCKET_NAME, &object_url)
        .ok_or_else(|| ProfileObjectCleanupFailure {
            profile_picture_id: profile_object.profile_picture_id,
            object_url: Some(object_url.clone()),
            reason: "profile metadata has an invalid object URL".to_string(),
            retryable: false,
        })?;
    match store.delete(location).await {
        Ok(()) => Ok(profile_object.profile_picture_id),
        Err(error) => {
            warn!(
                user_id = %user_id,
                profile_picture_id = %profile_object.profile_picture_id,
                retryable = error.is_retryable(),
                error = %error,
                "Profile object cleanup failed after hard purge"
            );
            Err(ProfileObjectCleanupFailure {
                profile_picture_id: profile_object.profile_picture_id,
                object_url: Some(object_url),
                reason: "object-store deletion failed".to_string(),
                retryable: error.is_retryable(),
            })
        }
    }
}
