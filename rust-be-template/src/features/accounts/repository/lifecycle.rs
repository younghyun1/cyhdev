//! Transactional account tombstoning and retained-identity purge.

use chrono::{DateTime, Utc};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;
use zeroize::Zeroizing;

use crate::{
    features::accounts::{
        domain::{
            account::DELETED_USER_DISPLAY_NAME,
            lifecycle::{
                AccountDeletionCandidate, RetainedAccountIdentity, SYSTEM_ACTOR_USER_ID,
                SoftDeleteAccountReceipt, TombstoneIdentity,
            },
            retention_notifications::RetentionNotificationSchedule,
        },
        error::AccountError,
        repository::{
            account_repository::AccountRepository,
            retention_notifications::insert_retention_notification_schedule,
        },
    },
    schema::{
        account_oidc_identities, deleted_account_retention, email_verification_tokens,
        forum_notifications, forum_topic_subscriptions, live_chat_call_participants,
        live_chat_messages, password_reset_tokens, photographs, user_roles, users,
    },
};

impl AccountRepository {
    /// Loads active credential state for self-service deletion confirmation.
    pub async fn account_deletion_candidate(
        &self,
        user_id: Uuid,
    ) -> Result<AccountDeletionCandidate, AccountError> {
        let mut connection = self.connection().await?;
        let record = users::table
            .filter(users::user_id.eq(user_id))
            .select((
                users::user_password_hash,
                users::user_is_system_actor,
                users::user_deleted_at,
            ))
            .first::<(String, bool, Option<DateTime<Utc>>)>(&mut connection)
            .await
            .optional()
            .map_err(AccountError::Query)?;
        let (password_hash, is_system_actor, deleted_at) = match record {
            Some(record) => record,
            None => return Err(AccountError::AccountNotFound),
        };
        if is_system_actor {
            return Err(AccountError::SystemActorProtected);
        }
        if deleted_at.is_some() {
            return Err(AccountError::AccountAlreadyDeleted);
        }

        Ok(AccountDeletionCandidate {
            user_id,
            password_hash: Zeroizing::new(password_hash),
            is_system_actor,
        })
    }

    /// Retains private identity and anonymizes the live account row atomically.
    pub async fn soft_delete_account(
        &self,
        user_id: Uuid,
        expected_password_hash: &str,
        deleted_at: DateTime<Utc>,
        purge_after: DateTime<Utc>,
        notification_schedule: RetentionNotificationSchedule,
    ) -> Result<SoftDeleteAccountReceipt, AccountError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<SoftDeleteAccountReceipt, AccountError, _>(async move |connection| {
                let locked_identity = users::table
                    .filter(users::user_id.eq(user_id))
                    .select((
                        users::user_name,
                        users::user_email,
                        users::user_country,
                        users::user_language,
                        users::user_subdivision,
                        users::user_password_hash,
                        users::user_is_system_actor,
                        users::user_deleted_at,
                    ))
                    .for_update()
                    .first::<(
                        String,
                        String,
                        i32,
                        i32,
                        Option<i32>,
                        String,
                        bool,
                        Option<DateTime<Utc>>,
                    )>(&mut *connection)
                    .await
                    .optional()?
                    .ok_or(AccountError::AccountNotFound)?;
                let (
                    user_name,
                    email,
                    country,
                    language,
                    subdivision,
                    stored_password_hash,
                    is_system_actor,
                    existing_deleted_at,
                ) = locked_identity;
                if is_system_actor {
                    return Err(AccountError::SystemActorProtected);
                }
                if existing_deleted_at.is_some() {
                    return Err(AccountError::AccountAlreadyDeleted);
                }
                if stored_password_hash != expected_password_hash {
                    return Err(AccountError::AccountChanged);
                }
                let retained = RetainedAccountIdentity {
                    user_id,
                    user_name,
                    email,
                    country,
                    language,
                    subdivision,
                };

                let (system_country, system_language) = users::table
                    .filter(users::user_id.eq(SYSTEM_ACTOR_USER_ID))
                    .filter(users::user_is_system_actor.eq(true))
                    .filter(users::user_deleted_at.is_null())
                    .select((users::user_country, users::user_language))
                    .first::<(i32, i32)>(&mut *connection)
                    .await
                    .optional()?
                    .ok_or(AccountError::SystemActorMissing)?;

                let retention_id = diesel::insert_into(deleted_account_retention::table)
                    .values((
                        deleted_account_retention::deleted_account_retention_user_id
                            .eq(retained.user_id),
                        deleted_account_retention::deleted_account_retention_user_name
                            .eq(&retained.user_name),
                        deleted_account_retention::deleted_account_retention_email
                            .eq(&retained.email),
                        deleted_account_retention::deleted_account_retention_country
                            .eq(retained.country),
                        deleted_account_retention::deleted_account_retention_language
                            .eq(retained.language),
                        deleted_account_retention::deleted_account_retention_subdivision
                            .eq(retained.subdivision),
                        deleted_account_retention::deleted_account_retention_created_at
                            .eq(deleted_at),
                    ))
                    .returning(
                        deleted_account_retention::deleted_account_retention_id,
                    )
                    .get_result::<Uuid>(&mut *connection)
                    .await?;

                clear_account_authority(connection, user_id).await?;
                anonymize_live_chat_history(connection, user_id).await?;
                anonymize_photograph_locations(connection, user_id).await?;

                let tombstone = TombstoneIdentity::for_retention_id(retention_id);
                diesel::update(users::table.filter(users::user_id.eq(user_id)))
                    .set((
                        users::user_name.eq(tombstone.user_name),
                        users::user_email.eq(tombstone.email),
                        users::user_password_hash.eq(tombstone.password_hash),
                        users::user_updated_at.eq(deleted_at),
                        users::user_is_email_verified.eq(false),
                        users::user_country.eq(system_country),
                        users::user_language.eq(system_language),
                        users::user_subdivision.eq(Option::<i32>::None),
                        users::user_deleted_at.eq(deleted_at),
                        users::user_purge_after.eq(purge_after),
                        users::user_hard_purged_at.eq(Option::<DateTime<Utc>>::None),
                    ))
                    .execute(&mut *connection)
                    .await?;

                insert_retention_notification_schedule(
                    connection,
                    user_id,
                    &notification_schedule,
                    deleted_at,
                )
                .await?;

                Ok(SoftDeleteAccountReceipt {
                    user_id,
                    deleted_at,
                    purge_after,
                })
            })
            .await
    }

}

async fn clear_account_authority(
    connection: &mut diesel_async::AsyncPgConnection,
    user_id: Uuid,
) -> Result<(), AccountError> {
    diesel::delete(
        forum_notifications::table.filter(
            forum_notifications::forum_notification_recipient_user_id.eq(user_id),
        ),
    )
    .execute(&mut *connection)
    .await?;
    diesel::delete(
        forum_topic_subscriptions::table.filter(
            forum_topic_subscriptions::forum_topic_subscription_user_id.eq(user_id),
        ),
    )
    .execute(&mut *connection)
    .await?;
    diesel::delete(
        account_oidc_identities::table.filter(
            account_oidc_identities::account_oidc_identity_user_id.eq(user_id),
        ),
    )
    .execute(&mut *connection)
    .await?;
    diesel::delete(user_roles::table.filter(user_roles::user_id.eq(user_id)))
        .execute(&mut *connection)
        .await?;
    diesel::delete(
        email_verification_tokens::table.filter(email_verification_tokens::user_id.eq(user_id)),
    )
    .execute(&mut *connection)
    .await?;
    diesel::delete(
        password_reset_tokens::table.filter(password_reset_tokens::user_id.eq(user_id)),
    )
    .execute(&mut *connection)
    .await?;
    Ok(())
}

pub(super) async fn anonymize_live_chat_history(
    connection: &mut diesel_async::AsyncPgConnection,
    user_id: Uuid,
) -> Result<(), AccountError> {
    diesel::update(live_chat_messages::table.filter(live_chat_messages::user_id.eq(user_id)))
        .set(live_chat_messages::sender_display_name.eq(DELETED_USER_DISPLAY_NAME))
        .execute(&mut *connection)
        .await?;
    diesel::update(
        live_chat_call_participants::table
            .filter(live_chat_call_participants::user_id.eq(user_id)),
    )
    .set(live_chat_call_participants::participant_display_name.eq(DELETED_USER_DISPLAY_NAME))
    .execute(&mut *connection)
    .await?;
    Ok(())
}

pub(super) async fn anonymize_photograph_locations(
    connection: &mut diesel_async::AsyncPgConnection,
    user_id: Uuid,
) -> Result<(), AccountError> {
    diesel::update(photographs::table.filter(photographs::user_id.eq(user_id)))
        .set((
            photographs::photograph_lat.eq(0.0_f64),
            photographs::photograph_lon.eq(0.0_f64),
        ))
        .execute(&mut *connection)
        .await?;
    Ok(())
}
