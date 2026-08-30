//! Profile-picture metadata persistence.

use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    features::accounts::{
        domain::account::ProfilePictureReplacement,
        error::AccountError,
        repository::{
            account_repository::AccountRepository,
            records::NewProfilePictureRecord,
        },
    },
    persistence::media_cleanup::enqueue_media_cleanup,
    schema::{user_profile_pictures, users},
    util::media::cleanup::{
        MediaCleanupRequest, REASON_PROFILE_PICTURE_HISTORY_PRUNED,
    },
};

pub const PROFILE_PICTURE_HISTORY_LIMIT: i64 = 8;

impl AccountRepository {
    /// Commits a new active picture and prunes overflow history in one transaction.
    pub async fn replace_profile_picture(
        &self,
        user_id: Uuid,
        image_type: i32,
        is_on_cloud: bool,
        link: Option<&str>,
    ) -> Result<ProfilePictureReplacement, AccountError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<ProfilePictureReplacement, diesel::result::Error, _>(
                async |connection| {
                    // Locking the account serializes even its first two picture uploads,
                    // when no profile-picture row exists to lock yet.
                    users::table
                        .find(user_id)
                        .filter(users::user_deleted_at.is_null())
                        .select(users::user_id)
                        .for_update()
                        .first::<Uuid>(&mut *connection)
                        .await?;

                    diesel::update(
                        user_profile_pictures::table
                            .filter(user_profile_pictures::user_id.eq(user_id))
                            .filter(user_profile_pictures::user_profile_picture_is_active.eq(true)),
                    )
                    .set(user_profile_pictures::user_profile_picture_is_active.eq(false))
                    .execute(&mut *connection)
                    .await?;

                    let profile_picture_id = diesel::insert_into(user_profile_pictures::table)
                        .values(NewProfilePictureRecord {
                            user_id,
                            user_profile_picture_image_type: image_type,
                            user_profile_picture_is_on_cloud: is_on_cloud,
                            user_profile_picture_is_active: is_on_cloud && link.is_some(),
                            user_profile_picture_link: link,
                        })
                        .returning(user_profile_pictures::user_profile_picture_id)
                        .get_result(&mut *connection)
                        .await?;

                    let overflow_rows = user_profile_pictures::table
                        .filter(user_profile_pictures::user_id.eq(user_id))
                        .order((
                            user_profile_pictures::user_profile_picture_created_at.desc(),
                            user_profile_pictures::user_profile_picture_id.desc(),
                        ))
                        .offset(PROFILE_PICTURE_HISTORY_LIMIT)
                        .select((
                            user_profile_pictures::user_profile_picture_id,
                            user_profile_pictures::user_profile_picture_link,
                        ))
                        .load::<(Uuid, Option<String>)>(&mut *connection)
                        .await?;
                    let overflow_ids = overflow_rows
                        .iter()
                        .map(|(overflow_id, _)| *overflow_id)
                        .collect::<Vec<_>>();
                    let cleanup = enqueue_media_cleanup(
                        connection,
                        overflow_rows
                            .into_iter()
                            .filter_map(|(source_id, original_url)| {
                                original_url.map(|original_url| MediaCleanupRequest {
                                    original_url,
                                    reason: REASON_PROFILE_PICTURE_HISTORY_PRUNED,
                                    source_id,
                                })
                            })
                            .collect(),
                    )
                    .await?;
                    if !overflow_ids.is_empty() {
                        diesel::delete(
                            user_profile_pictures::table.filter(
                                user_profile_pictures::user_profile_picture_id
                                    .eq_any(overflow_ids),
                            ),
                        )
                        .execute(&mut *connection)
                        .await?;
                    }

                    Ok(ProfilePictureReplacement {
                        profile_picture_id,
                        cleanup_objects: cleanup.resolved,
                        unresolved_cleanup_count: cleanup.unresolved_count,
                    })
                },
            )
            .await
            .map_err(AccountError::Mutation)
    }

}
