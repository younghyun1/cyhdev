//! Bounded photograph deletion with durable object cleanup.

use std::sync::Arc;

use axum::{Extension, Json, extract::State, response::IntoResponse};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::{AsyncConnection, RunQueryDsl};
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    dto::{
        requests::photography::delete_photographs_request::DeletePhotographsRequest,
        responses::{
            photography::delete_photographs_response::DeletePhotographsResponse,
            response_data::http_resp,
        },
    },
    errors::code_error::{CodeError, CodeErrorResp, HandlerResponse, code_err},
    features::accounts::repository::active_user::{ActiveUserWriteError, lock_active_superuser},
    init::state::ServerState,
    schema::photographs,
    util::{
        media::{
            cleanup::{
                MediaCleanupRequest, REASON_DELETED_PHOTOGRAPH_IMAGE,
                REASON_DELETED_PHOTOGRAPH_THUMBNAIL, enqueue_media_cleanup,
                settle_durable_cleanup,
            },
            object_store::S3MediaObjectStore,
            persistence::cleanup_committed_objects,
        },
        time::now::tokio_now,
    },
};

const MAX_DELETE_PHOTOGRAPHS: usize = 1_000;
const PHOTOGRAPH_CLEANUP_CONCURRENCY: usize = 8;

#[utoipa::path(
    delete,
    path = "/api/photographs/delete",
    tag = "photography",
    request_body = DeletePhotographsRequest,
    responses(
        (status = 200, description = "Photographs deleted with explicit object cleanup status", body = DeletePhotographsResponse),
        (status = 400, description = "Invalid or oversized deletion set", body = CodeErrorResp),
        (status = 401, description = "Unauthorized", body = CodeErrorResp),
        (status = 403, description = "Forbidden (not superuser)", body = CodeErrorResp),
        (status = 500, description = "Internal server error", body = CodeErrorResp)
    )
)]
pub async fn delete_photographs(
    Extension(requester_id): Extension<Uuid>,
    State(state): State<Arc<ServerState>>,
    Json(body): Json<DeletePhotographsRequest>,
) -> HandlerResponse<impl IntoResponse> {
    let start = tokio_now();
    let photograph_ids = normalize_photograph_ids(body.photograph_ids).map_err(|count| {
        code_err(
            CodeError::INVALID_REQUEST,
            format!(
                "at most {MAX_DELETE_PHOTOGRAPHS} distinct photograph ids may be deleted; received {count}"
            ),
        )
    })?;
    if photograph_ids.is_empty() {
        return Ok(http_resp(DeletePhotographsResponse::empty(), (), start));
    }

    let mut connection = state
        .get_conn()
        .await
        .map_err(|error| code_err(CodeError::POOL_ERROR, error))?;
    let (cleanup, deleted_rows) = match connection
        .transaction::<_, ActiveUserWriteError, _>(async |connection| {
            lock_active_superuser(&mut *connection, requester_id).await?;
            let targets = photographs::table
                .filter(photographs::photograph_id.eq_any(&photograph_ids))
                .select((
                    photographs::photograph_id,
                    photographs::photograph_link,
                    photographs::photograph_thumbnail_link,
                ))
                .load::<(Uuid, String, String)>(&mut *connection)
                .await?;
            let cleanup_requests = targets
                .into_iter()
                .flat_map(|(source_id, image_url, thumbnail_url)| {
                    [
                        MediaCleanupRequest {
                            original_url: image_url,
                            reason: REASON_DELETED_PHOTOGRAPH_IMAGE,
                            source_id,
                        },
                        MediaCleanupRequest {
                            original_url: thumbnail_url,
                            reason: REASON_DELETED_PHOTOGRAPH_THUMBNAIL,
                            source_id,
                        },
                    ]
                })
                .collect();
            let cleanup = enqueue_media_cleanup(connection, cleanup_requests).await?;
            let deleted_rows = diesel::delete(
                photographs::table.filter(photographs::photograph_id.eq_any(&photograph_ids)),
            )
            .execute(&mut *connection)
            .await?;
            Ok((cleanup, deleted_rows))
        })
        .await
    {
        Ok(result) => result,
        Err(ActiveUserWriteError::Inactive | ActiveUserWriteError::Denied) => {
            return Err(CodeError::UNAUTHORIZED_ACCESS.into());
        }
        Err(ActiveUserWriteError::Database(error)) => {
            return Err(code_err(CodeError::DB_DELETION_ERROR, error));
        }
        Err(error) => return Err(code_err(CodeError::DB_DELETION_ERROR, error)),
    };
    drop(connection);

    let cleanup_total = cleanup.resolved.len() + cleanup.unresolved_count;
    let locations = cleanup
        .resolved
        .iter()
        .map(|cleanup| cleanup.location.clone())
        .collect();
    let store = S3MediaObjectStore::from_config(&state.aws_profile_picture_config);
    let (cleaned, failures) = cleanup_committed_objects(
        &store,
        locations,
        PHOTOGRAPH_CLEANUP_CONCURRENCY,
    )
    .await;
    for failure in &failures {
        error!(
            bucket = %failure.location.bucket(),
            key = %failure.location.key(),
            retryable = failure.is_retryable(),
            error = %failure.error,
            "Photograph object cleanup remains pending"
        );
    }
    let settlement = settle_durable_cleanup(
        &state.account_service(),
        cleanup.resolved,
        &cleaned,
        &failures,
    )
    .await;
    let response = DeletePhotographsResponse {
        deleted_count: deleted_rows,
        s3_deleted_count: cleaned.len(),
        cleanup_failure_count: failures.len() + settlement.ledger_errors,
        cleanup_remaining_count: cleanup_total.saturating_sub(settlement.finalized),
        unresolved_cleanup_count: cleanup.unresolved_count,
    };
    info!(
        deleted_db_rows = response.deleted_count,
        deleted_objects = response.s3_deleted_count,
        cleanup_failures = response.cleanup_failure_count,
        cleanup_remaining = response.cleanup_remaining_count,
        "Completed bounded photograph deletion"
    );
    Ok(http_resp(response, (), start))
}

fn normalize_photograph_ids(mut photograph_ids: Vec<Uuid>) -> Result<Vec<Uuid>, usize> {
    photograph_ids.sort_unstable();
    photograph_ids.dedup();
    if photograph_ids.len() > MAX_DELETE_PHOTOGRAPHS {
        return Err(photograph_ids.len());
    }
    Ok(photograph_ids)
}

#[cfg(test)]
mod tests {
    use super::{MAX_DELETE_PHOTOGRAPHS, normalize_photograph_ids};
    use uuid::Uuid;

    #[test]
    fn photograph_deletion_deduplicates_before_enforcing_the_cap() {
        let repeated = vec![Uuid::from_u128(1); MAX_DELETE_PHOTOGRAPHS + 1];
        let normalized = normalize_photograph_ids(repeated);
        assert!(matches!(normalized, Ok(ids) if ids.len() == 1));
    }

    #[test]
    fn photograph_deletion_rejects_too_many_distinct_ids() {
        let ids = (0..=MAX_DELETE_PHOTOGRAPHS)
            .map(|value| Uuid::from_u128(value as u128))
            .collect();
        assert!(matches!(
            normalize_photograph_ids(ids),
            Err(count) if count == MAX_DELETE_PHOTOGRAPHS + 1
        ));
    }
}
