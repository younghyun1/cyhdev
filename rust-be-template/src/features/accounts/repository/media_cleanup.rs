//! Durable unresolved-media reconciliation with database-authoritative authorization.

use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    features::accounts::{
        domain::lifecycle::{
            MEDIA_CLEANUP_RECONCILIATION_LIMIT, ResolvedMediaCleanup, UnresolvedMediaCleanup,
        },
        error::AccountError,
        repository::account_repository::AccountRepository,
    },
    schema::media_object_cleanup,
};

use super::hard_purge::lock_hard_purge_requester;

const RESOLVED_OBJECT_UNIQUE_CONSTRAINT: &str = "media_object_cleanup_resolved_object_unique";

impl AccountRepository {
    pub async fn unresolved_media_cleanup(
        &self,
        requester_id: Uuid,
    ) -> Result<Vec<UnresolvedMediaCleanup>, AccountError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<Vec<UnresolvedMediaCleanup>, AccountError, _>(async move |connection| {
                lock_hard_purge_requester(connection, requester_id).await?;
                let rows = media_object_cleanup::table
                    .filter(media_object_cleanup::media_object_cleanup_bucket.is_null())
                    .filter(media_object_cleanup::media_object_cleanup_key.is_null())
                    .order((
                        media_object_cleanup::media_object_cleanup_created_at.asc(),
                        media_object_cleanup::media_object_cleanup_id.asc(),
                    ))
                    .limit(MEDIA_CLEANUP_RECONCILIATION_LIMIT)
                    .select((
                        media_object_cleanup::media_object_cleanup_id,
                        media_object_cleanup::media_object_cleanup_original_url,
                        media_object_cleanup::media_object_cleanup_reason,
                        media_object_cleanup::media_object_cleanup_source_id,
                        media_object_cleanup::media_object_cleanup_created_at,
                    ))
                    .load::<(Uuid, String, String, Uuid, chrono::DateTime<chrono::Utc>)>(
                        &mut *connection,
                    )
                    .await?;
                Ok(rows
                    .into_iter()
                    .map(
                        |(cleanup_id, original_url, reason, source_id, created_at)| {
                            UnresolvedMediaCleanup {
                                cleanup_id,
                                original_url,
                                reason,
                                source_id,
                                created_at,
                            }
                        },
                    )
                    .collect())
            })
            .await
    }

    pub async fn resolve_media_cleanup(
        &self,
        requester_id: Uuid,
        cleanup_id: Uuid,
        expected_original_url: &str,
        bucket: &str,
        key: &str,
    ) -> Result<ResolvedMediaCleanup, AccountError> {
        let mut connection = self.connection().await?;
        let result = connection
            .transaction::<ResolvedMediaCleanup, AccountError, _>(async move |connection| {
                lock_hard_purge_requester(connection, requester_id).await?;
                let stored = media_object_cleanup::table
                    .filter(media_object_cleanup::media_object_cleanup_id.eq(cleanup_id))
                    .select((
                        media_object_cleanup::media_object_cleanup_bucket,
                        media_object_cleanup::media_object_cleanup_key,
                        media_object_cleanup::media_object_cleanup_original_url,
                    ))
                    .for_update()
                    .first::<(Option<String>, Option<String>, String)>(&mut *connection)
                    .await
                    .optional()?
                    .ok_or(AccountError::MediaCleanupNotFound)?;
                let (stored_bucket, stored_key, original_url) = stored;
                if original_url != expected_original_url {
                    return Err(AccountError::MediaCleanupOriginalUrlMismatch);
                }
                match (stored_bucket, stored_key) {
                    (Some(stored_bucket), Some(stored_key)) => {
                        if stored_bucket == bucket && stored_key == key {
                            return Ok(ResolvedMediaCleanup {
                                cleanup_id,
                                bucket: stored_bucket,
                                key: stored_key,
                                original_url,
                            });
                        }
                        return Err(AccountError::MediaCleanupAlreadyResolved);
                    }
                    (None, None) => {}
                    _ => return Err(AccountError::InvalidMediaCleanupLocation),
                }

                diesel::update(
                    media_object_cleanup::table
                        .filter(media_object_cleanup::media_object_cleanup_id.eq(cleanup_id)),
                )
                .set((
                    media_object_cleanup::media_object_cleanup_bucket.eq(bucket),
                    media_object_cleanup::media_object_cleanup_key.eq(key),
                ))
                .execute(&mut *connection)
                .await?;
                Ok(ResolvedMediaCleanup {
                    cleanup_id,
                    bucket: bucket.to_owned(),
                    key: key.to_owned(),
                    original_url,
                })
            })
            .await;
        result.map_err(classify_resolution_error)
    }
}

fn classify_resolution_error(error: AccountError) -> AccountError {
    let is_object_conflict = match &error {
        AccountError::Mutation(diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::UniqueViolation,
            information,
        )) => information.constraint_name() == Some(RESOLVED_OBJECT_UNIQUE_CONSTRAINT),
        _ => false,
    };
    if is_object_conflict {
        AccountError::MediaCleanupObjectConflict
    } else {
        error
    }
}
