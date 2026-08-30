//! Irreversible retained-identity and profile-metadata purge.

use chrono::{DateTime, Utc};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    features::accounts::{
        domain::lifecycle::{
            HardPurgeAccountPlan, ProfileCleanupFinalization, ProfileObjectCleanup,
            PROFILE_CLEANUP_BATCH_SIZE,
        },
        domain::role::RoleType,
        error::AccountError,
        repository::account_repository::AccountRepository,
    },
    schema::{deleted_account_retention, user_profile_pictures, user_roles, users},
};

use super::lifecycle::{anonymize_live_chat_history, anonymize_photograph_locations};

impl AccountRepository {
    /// Deletes retained private identity while preserving the authored-content tombstone.
    pub async fn hard_purge_account(
        &self,
        requester_id: Uuid,
        user_id: Uuid,
        hard_purged_at: DateTime<Utc>,
    ) -> Result<HardPurgeAccountPlan, AccountError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<HardPurgeAccountPlan, AccountError, _>(async move |connection| {
                lock_hard_purge_requester(connection, requester_id).await?;
                let lifecycle = users::table
                    .filter(users::user_id.eq(user_id))
                    .select((
                        users::user_deleted_at,
                        users::user_purge_after,
                        users::user_hard_purged_at,
                        users::user_is_system_actor,
                    ))
                    .for_update()
                    .first::<(
                        Option<DateTime<Utc>>,
                        Option<DateTime<Utc>>,
                        Option<DateTime<Utc>>,
                        bool,
                    )>(&mut *connection)
                    .await
                    .optional()?
                    .ok_or(AccountError::AccountNotFound)?;
                let (deleted_at, purge_after, previous_hard_purge, is_system_actor) = lifecycle;
                if is_system_actor {
                    return Err(AccountError::SystemActorProtected);
                }
                if deleted_at.is_none() {
                    return Err(AccountError::AccountNotDeleted);
                }
                let purge_after = purge_after.ok_or(AccountError::AccountNotDeleted)?;
                let effective_hard_purged_at = match previous_hard_purge {
                    Some(previous_hard_purge) => {
                        diesel::delete(
                            deleted_account_retention::table.filter(
                                deleted_account_retention::deleted_account_retention_user_id
                                    .eq(user_id),
                            ),
                        )
                        .execute(&mut *connection)
                        .await?;
                        previous_hard_purge
                    }
                    None => {
                        if hard_purged_at < purge_after {
                            return Err(AccountError::RetentionPeriodActive { purge_after });
                        }
                        anonymize_live_chat_history(connection, user_id).await?;
                        anonymize_photograph_locations(connection, user_id).await?;
                        let deleted_retention = diesel::delete(
                            deleted_account_retention::table.filter(
                                deleted_account_retention::deleted_account_retention_user_id
                                    .eq(user_id),
                            ),
                        )
                        .execute(&mut *connection)
                        .await?;
                        if deleted_retention != 1 {
                            return Err(AccountError::RetainedIdentityMissing);
                        }
                        diesel::update(users::table.filter(users::user_id.eq(user_id)))
                            .set((
                                users::user_hard_purged_at.eq(hard_purged_at),
                                users::user_updated_at.eq(hard_purged_at),
                            ))
                            .execute(&mut *connection)
                            .await?;
                        hard_purged_at
                    }
                };

                let profile_rows = user_profile_pictures::table
                    .filter(user_profile_pictures::user_id.eq(user_id))
                    .order((
                        user_profile_pictures::user_profile_picture_created_at.asc(),
                        user_profile_pictures::user_profile_picture_id.asc(),
                    ))
                    .limit(PROFILE_CLEANUP_BATCH_SIZE)
                    .select((
                        user_profile_pictures::user_profile_picture_id,
                        user_profile_pictures::user_profile_picture_is_on_cloud,
                        user_profile_pictures::user_profile_picture_link,
                    ))
                    .load::<(Uuid, bool, Option<String>)>(&mut *connection)
                    .await?;
                let mut profile_objects = Vec::new();
                let mut non_cloud_profile_ids = Vec::new();
                for (profile_picture_id, is_on_cloud, object_url) in profile_rows {
                    if is_on_cloud {
                        profile_objects.push(ProfileObjectCleanup {
                            profile_picture_id,
                            object_url,
                        });
                    } else {
                        non_cloud_profile_ids.push(profile_picture_id);
                    }
                }

                Ok(HardPurgeAccountPlan {
                    user_id,
                    hard_purged_at: effective_hard_purged_at,
                    profile_objects,
                    non_cloud_profile_ids,
                })
            })
            .await
    }

    /// Removes profile metadata only after its remote object is gone or no object exists.
    pub async fn finalize_profile_cleanup(
        &self,
        requester_id: Uuid,
        user_id: Uuid,
        profile_picture_ids: &[Uuid],
    ) -> Result<ProfileCleanupFinalization, AccountError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<ProfileCleanupFinalization, AccountError, _>(async move |connection| {
                lock_hard_purge_requester(connection, requester_id).await?;
                let (hard_purged_at, is_system_actor) = users::table
                    .filter(users::user_id.eq(user_id))
                    .select((users::user_hard_purged_at, users::user_is_system_actor))
                    .for_update()
                    .first::<(Option<DateTime<Utc>>, bool)>(&mut *connection)
                    .await
                    .optional()?
                    .ok_or(AccountError::AccountNotFound)?;
                if is_system_actor {
                    return Err(AccountError::SystemActorProtected);
                }
                if hard_purged_at.is_none() {
                    return Err(AccountError::AccountNotHardPurged);
                }
                let metadata_deleted = if profile_picture_ids.is_empty() {
                    0
                } else {
                    diesel::delete(
                        user_profile_pictures::table
                            .filter(user_profile_pictures::user_id.eq(user_id))
                            .filter(
                                user_profile_pictures::user_profile_picture_id
                                    .eq_any(profile_picture_ids),
                            ),
                    )
                    .execute(&mut *connection)
                    .await?
                };
                let metadata_remaining = user_profile_pictures::table
                    .filter(user_profile_pictures::user_id.eq(user_id))
                    .count()
                    .get_result::<i64>(&mut *connection)
                    .await?;
                let metadata_remaining = usize::try_from(metadata_remaining)
                    .map_err(|_| AccountError::ProfileCleanupCountOverflow)?;
                Ok(ProfileCleanupFinalization {
                    metadata_deleted,
                    metadata_remaining,
                })
            })
            .await
    }
}

pub(super) async fn lock_hard_purge_requester(
    connection: &mut diesel_async::AsyncPgConnection,
    requester_id: Uuid,
) -> Result<(), AccountError> {
    let requester = users::table
        .filter(users::user_id.eq(requester_id))
        .filter(users::user_deleted_at.is_null())
        .select(users::user_id)
        .for_update()
        .first::<Uuid>(&mut *connection)
        .await
        .optional()?;
    if requester.is_none() {
        return Err(AccountError::HardPurgeRequesterUnauthorized);
    }
    let role_id = user_roles::table
        .filter(user_roles::user_id.eq(requester_id))
        .select(user_roles::role_id)
        .first::<Uuid>(&mut *connection)
        .await
        .optional()?;
    match role_id.and_then(RoleType::from_uuid) {
        Some(RoleType::Younghyun) => Ok(()),
        _ => Err(AccountError::HardPurgeRequesterUnauthorized),
    }
}
