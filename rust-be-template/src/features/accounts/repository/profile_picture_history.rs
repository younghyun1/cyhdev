//! Bounded profile-picture history selection and deletion.

use chrono::Utc;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    features::accounts::{
        domain::account::{ProfilePicture, ProfilePictureDeletion},
        error::AccountError,
        repository::{
            account_repository::AccountRepository, records::ProfilePictureRecord,
        },
    },
    schema::{user_profile_pictures, users},
    util::media::cleanup::{
        MediaCleanupRequest, REASON_PROFILE_PICTURE_DELETED, enqueue_media_cleanup,
    },
};

use super::profile_pictures::PROFILE_PICTURE_HISTORY_LIMIT;

impl AccountRepository {
    /// Lists the bounded valid history, newest first.
    pub async fn profile_picture_history(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<ProfilePicture>, AccountError> {
        let mut connection = self.connection().await?;
        user_profile_pictures::table
            .inner_join(users::table)
            .filter(user_profile_pictures::user_id.eq(user_id))
            .filter(users::user_deleted_at.is_null())
            .filter(user_profile_pictures::user_profile_picture_is_on_cloud.eq(true))
            .filter(user_profile_pictures::user_profile_picture_link.is_not_null())
            .order((
                user_profile_pictures::user_profile_picture_created_at.desc(),
                user_profile_pictures::user_profile_picture_id.desc(),
            ))
            .limit(PROFILE_PICTURE_HISTORY_LIMIT)
            .select(ProfilePictureRecord::as_select())
            .load::<ProfilePictureRecord>(&mut connection)
            .await
            .map(|records| records.into_iter().map(Into::into).collect())
            .map_err(AccountError::Query)
    }

    /// Atomically selects one valid owned history entry as active.
    pub async fn select_profile_picture(
        &self,
        user_id: Uuid,
        profile_picture_id: Uuid,
    ) -> Result<Option<ProfilePicture>, AccountError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<Option<ProfilePicture>, diesel::result::Error, _>(
                async |connection| {
                    lock_active_account(connection, user_id).await?;
                    let target = user_profile_pictures::table
                        .filter(user_profile_pictures::user_id.eq(user_id))
                        .filter(
                            user_profile_pictures::user_profile_picture_id
                                .eq(profile_picture_id),
                        )
                        .filter(
                            user_profile_pictures::user_profile_picture_is_on_cloud.eq(true),
                        )
                        .filter(user_profile_pictures::user_profile_picture_link.is_not_null())
                        .select(ProfilePictureRecord::as_select())
                        .first::<ProfilePictureRecord>(&mut *connection)
                        .await
                        .optional()?;
                    let Some(target) = target else {
                        return Ok(None);
                    };
                    let target: ProfilePicture = target.into();
                    if target.is_active {
                        return Ok(Some(target));
                    }

                    diesel::update(
                        user_profile_pictures::table
                            .filter(user_profile_pictures::user_id.eq(user_id))
                            .filter(user_profile_pictures::user_profile_picture_is_active.eq(true)),
                    )
                    .set(user_profile_pictures::user_profile_picture_is_active.eq(false))
                    .execute(&mut *connection)
                    .await?;
                    let selected = diesel::update(
                        user_profile_pictures::table.filter(
                            user_profile_pictures::user_profile_picture_id
                                .eq(profile_picture_id),
                        ),
                    )
                    .set((
                        user_profile_pictures::user_profile_picture_is_active.eq(true),
                        user_profile_pictures::user_profile_picture_updated_at.eq(Utc::now()),
                    ))
                    .returning(ProfilePictureRecord::as_returning())
                    .get_result::<ProfilePictureRecord>(&mut *connection)
                    .await?;
                    Ok(Some(selected.into()))
                },
            )
            .await
            .map_err(AccountError::Mutation)
    }

    /// Enqueues the object before deleting owned metadata and elects a new active row.
    pub async fn delete_profile_picture(
        &self,
        user_id: Uuid,
        profile_picture_id: Uuid,
    ) -> Result<Option<ProfilePictureDeletion>, AccountError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<Option<ProfilePictureDeletion>, diesel::result::Error, _>(
                async |connection| {
                    lock_active_account(connection, user_id).await?;
                    let target = user_profile_pictures::table
                        .filter(user_profile_pictures::user_id.eq(user_id))
                        .filter(
                            user_profile_pictures::user_profile_picture_id
                                .eq(profile_picture_id),
                        )
                        .select((
                            user_profile_pictures::user_profile_picture_is_active,
                            user_profile_pictures::user_profile_picture_link,
                        ))
                        .first::<(bool, Option<String>)>(&mut *connection)
                        .await
                        .optional()?;
                    let Some((was_active, original_url)) = target else {
                        return Ok(None);
                    };

                    let cleanup = enqueue_media_cleanup(
                        connection,
                        original_url
                            .map(|original_url| {
                                vec![MediaCleanupRequest {
                                    original_url,
                                    reason: REASON_PROFILE_PICTURE_DELETED,
                                    source_id: profile_picture_id,
                                }]
                            })
                            .unwrap_or_default(),
                    )
                    .await?;
                    diesel::delete(
                        user_profile_pictures::table.filter(
                            user_profile_pictures::user_profile_picture_id
                                .eq(profile_picture_id),
                        ),
                    )
                    .execute(&mut *connection)
                    .await?;

                    let active_profile_picture_id = if was_active {
                        let replacement = newest_valid_profile_picture(connection, user_id).await?;
                        if let Some(replacement_id) = replacement {
                            diesel::update(user_profile_pictures::table.find(replacement_id))
                                .set((
                                    user_profile_pictures::user_profile_picture_is_active.eq(true),
                                    user_profile_pictures::user_profile_picture_updated_at
                                        .eq(Utc::now()),
                                ))
                                .execute(&mut *connection)
                                .await?;
                        }
                        replacement
                    } else {
                        current_profile_picture(connection, user_id).await?
                    };

                    Ok(Some(ProfilePictureDeletion {
                        deleted_profile_picture_id: profile_picture_id,
                        active_profile_picture_id,
                        cleanup_objects: cleanup.resolved,
                        unresolved_cleanup_count: cleanup.unresolved_count,
                    }))
                },
            )
            .await
            .map_err(AccountError::Mutation)
    }
}

async fn lock_active_account(
    connection: &mut diesel_async::AsyncPgConnection,
    user_id: Uuid,
) -> diesel::result::QueryResult<()> {
    users::table
        .find(user_id)
        .filter(users::user_deleted_at.is_null())
        .select(users::user_id)
        .for_update()
        .first::<Uuid>(connection)
        .await
        .map(|_| ())
}

async fn newest_valid_profile_picture(
    connection: &mut diesel_async::AsyncPgConnection,
    user_id: Uuid,
) -> diesel::result::QueryResult<Option<Uuid>> {
    user_profile_pictures::table
        .filter(user_profile_pictures::user_id.eq(user_id))
        .filter(user_profile_pictures::user_profile_picture_is_on_cloud.eq(true))
        .filter(user_profile_pictures::user_profile_picture_link.is_not_null())
        .order((
            user_profile_pictures::user_profile_picture_created_at.desc(),
            user_profile_pictures::user_profile_picture_id.desc(),
        ))
        .select(user_profile_pictures::user_profile_picture_id)
        .first::<Uuid>(connection)
        .await
        .optional()
}

async fn current_profile_picture(
    connection: &mut diesel_async::AsyncPgConnection,
    user_id: Uuid,
) -> diesel::result::QueryResult<Option<Uuid>> {
    user_profile_pictures::table
        .filter(user_profile_pictures::user_id.eq(user_id))
        .filter(user_profile_pictures::user_profile_picture_is_active.eq(true))
        .select(user_profile_pictures::user_profile_picture_id)
        .first::<Uuid>(connection)
        .await
        .optional()
}
