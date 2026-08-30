//! Account-role persistence.

use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, dsl::exists};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    features::accounts::{
        domain::role::RoleType,
        error::AccountError,
        repository::{account_repository::AccountRepository, records::NewUserRoleRecord},
    },
    schema::user_roles,
};

impl AccountRepository {
    pub async fn assign_role(&self, user_id: Uuid, role: RoleType) -> Result<(), AccountError> {
        let mut connection = self.connection().await?;
        diesel::insert_into(user_roles::table)
            .values(NewUserRoleRecord {
                user_id,
                role_id: role.id(),
            })
            .on_conflict(user_roles::user_id)
            .do_update()
            .set(user_roles::role_id.eq(role.id()))
            .execute(&mut connection)
            .await
            .map(|_| ())
            .map_err(AccountError::Mutation)
    }

    pub async fn role_for_user(&self, user_id: Uuid) -> Result<Option<RoleType>, AccountError> {
        let mut connection = self.connection().await?;
        let role_id = user_roles::table
            .filter(user_roles::user_id.eq(user_id))
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
        let existing_role_id = user_roles::table
            .filter(user_roles::user_id.eq(user_id))
            .select(user_roles::role_id)
            .first::<Uuid>(&mut connection)
            .await
            .optional()
            .map_err(AccountError::Query)?;

        if let Some(role_id) = existing_role_id {
            return RoleType::from_uuid(role_id).ok_or(AccountError::InvalidRoleId(role_id));
        }

        let insert_result = diesel::insert_into(user_roles::table)
            .values(NewUserRoleRecord {
                user_id,
                role_id: default_role.id(),
            })
            .execute(&mut connection)
            .await;

        match insert_result {
            Ok(_) => Ok(default_role),
            Err(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            )) => {
                let role_id = user_roles::table
                    .filter(user_roles::user_id.eq(user_id))
                    .select(user_roles::role_id)
                    .first::<Uuid>(&mut connection)
                    .await
                    .map_err(AccountError::Query)?;
                RoleType::from_uuid(role_id).ok_or(AccountError::InvalidRoleId(role_id))
            }
            Err(error) => Err(AccountError::Mutation(error)),
        }
    }

    pub async fn has_role(&self, user_id: Uuid, role: RoleType) -> Result<bool, AccountError> {
        let mut connection = self.connection().await?;
        let matching_role = user_roles::table
            .filter(user_roles::user_id.eq(user_id))
            .filter(user_roles::role_id.eq(role.id()));

        diesel::select(exists(matching_role))
            .get_result::<bool>(&mut connection)
            .await
            .map_err(AccountError::Query)
    }
}
