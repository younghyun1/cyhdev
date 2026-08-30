//! Account-role persistence.

use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, dsl::exists};
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    features::accounts::{
        domain::role::RoleType,
        error::AccountError,
        repository::{account_repository::AccountRepository, records::NewUserRoleRecord},
    },
    schema::{user_roles, users},
};

impl AccountRepository {
    pub async fn assign_role(&self, user_id: Uuid, role: RoleType) -> Result<(), AccountError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<(), AccountError, _>(async move |connection| {
                lock_active_account(connection, user_id).await?;
                diesel::insert_into(user_roles::table)
                    .values(NewUserRoleRecord {
                        user_id,
                        role_id: role.id(),
                    })
                    .on_conflict(user_roles::user_id)
                    .do_update()
                    .set(user_roles::role_id.eq(role.id()))
                    .execute(&mut *connection)
                    .await?;
                Ok(())
            })
            .await
    }

    pub async fn role_for_user(&self, user_id: Uuid) -> Result<Option<RoleType>, AccountError> {
        let mut connection = self.connection().await?;
        let role_id = user_roles::table
            .inner_join(users::table)
            .filter(user_roles::user_id.eq(user_id))
            .filter(users::user_deleted_at.is_null())
            .select(user_roles::role_id)
            .first::<Uuid>(&mut connection)
            .await
            .optional()
            .map_err(AccountError::Query)?;

        match role_id {
            Some(role_id) => RoleType::from_uuid(role_id)
                .map(Some)
                .ok_or(AccountError::InvalidRoleId(role_id)),
            None => Ok(None),
        }
    }

    pub async fn role_for_user_or_insert_default(
        &self,
        user_id: Uuid,
        default_role: RoleType,
    ) -> Result<RoleType, AccountError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<RoleType, AccountError, _>(async move |connection| {
                lock_active_account(connection, user_id).await?;
                let existing_role_id = user_roles::table
                    .filter(user_roles::user_id.eq(user_id))
                    .select(user_roles::role_id)
                    .first::<Uuid>(&mut *connection)
                    .await
                    .optional()?;
                if let Some(role_id) = existing_role_id {
                    return RoleType::from_uuid(role_id)
                        .ok_or(AccountError::InvalidRoleId(role_id));
                }
                diesel::insert_into(user_roles::table)
                    .values(NewUserRoleRecord {
                        user_id,
                        role_id: default_role.id(),
                    })
                    .execute(&mut *connection)
                    .await?;
                Ok(default_role)
            })
            .await
    }

    pub async fn has_role(&self, user_id: Uuid, role: RoleType) -> Result<bool, AccountError> {
        let mut connection = self.connection().await?;
        let matching_role = user_roles::table
            .inner_join(users::table)
            .filter(user_roles::user_id.eq(user_id))
            .filter(users::user_deleted_at.is_null())
            .filter(user_roles::role_id.eq(role.id()));

        diesel::select(exists(matching_role))
            .get_result::<bool>(&mut connection)
            .await
            .map_err(AccountError::Query)
    }
}

async fn lock_active_account(
    connection: &mut diesel_async::AsyncPgConnection,
    user_id: Uuid,
) -> Result<(), AccountError> {
    let active_user = users::table
        .filter(users::user_id.eq(user_id))
        .filter(users::user_deleted_at.is_null())
        .select(users::user_id)
        .for_update()
        .first::<Uuid>(&mut *connection)
        .await
        .optional()?;
    match active_user {
        Some(_) => Ok(()),
        None => Err(AccountError::AccountNotFound),
    }
}
