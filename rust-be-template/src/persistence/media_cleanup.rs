//! Transaction-scoped durable cleanup registration for media repositories.

use diesel::{ExpressionMethods, NullableExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    schema::media_object_cleanup,
    util::{
        media::{
            cleanup::{DurableMediaCleanup, EnqueuedMediaCleanup, MediaCleanupRequest},
            object_store::ObjectLocation,
        },
        s3::AWS_S3_BUCKET_NAME,
    },
};

#[derive(diesel::Insertable)]
#[diesel(table_name = media_object_cleanup)]
struct NewMediaObjectCleanup {
    media_object_cleanup_bucket: Option<String>,
    media_object_cleanup_key: Option<String>,
    media_object_cleanup_original_url: String,
    media_object_cleanup_reason: &'static str,
    media_object_cleanup_source_id: Uuid,
}

/// Enqueues cleanup in the caller's metadata transaction.
pub async fn enqueue_media_cleanup(
    connection: &mut diesel_async::AsyncPgConnection,
    requests: Vec<MediaCleanupRequest>,
) -> diesel::result::QueryResult<EnqueuedMediaCleanup> {
    if requests.is_empty() {
        return Ok(EnqueuedMediaCleanup {
            resolved: Vec::new(),
            unresolved_count: 0,
        });
    }

    let mut rows = Vec::with_capacity(requests.len());
    let mut resolved_keys = Vec::with_capacity(requests.len());
    let mut unresolved_count = 0_usize;
    for request in requests {
        let location =
            ObjectLocation::from_public_s3_url(AWS_S3_BUCKET_NAME, &request.original_url);
        let (bucket, key) = match location {
            Some(location) => {
                resolved_keys.push(location.key().to_owned());
                (
                    Some(location.bucket().to_owned()),
                    Some(location.key().to_owned()),
                )
            }
            None => {
                unresolved_count = unresolved_count.saturating_add(1);
                (None, None)
            }
        };
        rows.push(NewMediaObjectCleanup {
            media_object_cleanup_bucket: bucket,
            media_object_cleanup_key: key,
            media_object_cleanup_original_url: request.original_url,
            media_object_cleanup_reason: request.reason,
            media_object_cleanup_source_id: request.source_id,
        });
    }

    diesel::insert_into(media_object_cleanup::table)
        .values(&rows)
        .on_conflict_do_nothing()
        .execute(&mut *connection)
        .await?;

    if resolved_keys.is_empty() {
        return Ok(EnqueuedMediaCleanup {
            resolved: Vec::new(),
            unresolved_count,
        });
    }
    resolved_keys.sort_unstable();
    resolved_keys.dedup();
    let resolved = media_object_cleanup::table
        .filter(media_object_cleanup::media_object_cleanup_bucket.eq(AWS_S3_BUCKET_NAME))
        .filter(media_object_cleanup::media_object_cleanup_key.eq_any(resolved_keys))
        .filter(media_object_cleanup::media_object_cleanup_bucket.is_not_null())
        .filter(media_object_cleanup::media_object_cleanup_key.is_not_null())
        .select((
            media_object_cleanup::media_object_cleanup_id,
            media_object_cleanup::media_object_cleanup_bucket.assume_not_null(),
            media_object_cleanup::media_object_cleanup_key.assume_not_null(),
            media_object_cleanup::media_object_cleanup_attempt_count,
        ))
        .load::<(Uuid, String, String, i32)>(&mut *connection)
        .await?
        .into_iter()
        .map(
            |(cleanup_id, bucket, key, attempt_count)| DurableMediaCleanup {
                cleanup_id,
                location: ObjectLocation::new(bucket, key),
                attempt_count,
            },
        )
        .collect();
    Ok(EnqueuedMediaCleanup {
        resolved,
        unresolved_count,
    })
}
