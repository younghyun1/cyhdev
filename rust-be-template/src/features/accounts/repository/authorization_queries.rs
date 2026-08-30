//! Bounded, database-current authorization administration reads.

use diesel::{
    BoolExpressionMethods, EscapeExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl,
    TextExpressionMethods,
};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    features::accounts::{
        authorization_error::AuthorizationError,
        domain::{
            authorization::{
                AuthorizationPage, AuthorizationPageSize, AuthorizationSearch, AuthorizationUser,
                MAX_AUTHORIZATION_CATALOG_ROWS, PermissionDefinition, PermissionName,
                RoleDefinition, RolePermissionBinding,
            },
            role::RoleType,
        },
        repository::{
            account_repository::AccountRepository,
            authorization_guard::ensure_current_younghyun,
            authorization_records::{
                AuthorizationUserRow, PermissionDefinitionRow, RoleDefinitionRow,
                RolePermissionBindingRow,
            },
        },
    },
    schema::{permissions, role_permissions, roles, user_roles, users},
};

impl AccountRepository {
    pub async fn has_current_permission(
        &self,
        user_id: Uuid,
        permission_name: &PermissionName,
    ) -> Result<bool, AuthorizationError> {
        let mut connection = self.connection().await?;
        let current_authority = user_roles::table
            .inner_join(users::table)
            .inner_join(
                role_permissions::table
                    .on(role_permissions::role_id.eq(user_roles::role_id)),
            )
            .inner_join(
                permissions::table
                    .on(permissions::permission_id.eq(role_permissions::permission_id)),
            )
            .filter(user_roles::user_id.eq(user_id))
            .filter(users::user_deleted_at.is_null())
            .filter(users::user_hard_purged_at.is_null())
            .filter(permissions::permission_name.eq(permission_name.as_ref()));
        diesel::select(diesel::dsl::exists(current_authority))
            .get_result::<bool>(&mut connection)
            .await
            .map_err(AuthorizationError::Query)
    }

    pub async fn authorization_users(
        &self,
        actor_user_id: Uuid,
        search: Option<&AuthorizationSearch>,
        after: Option<Uuid>,
        page_size: AuthorizationPageSize,
    ) -> Result<AuthorizationPage<AuthorizationUser>, AuthorizationError> {
        let mut connection = self.connection().await?;
        ensure_current_younghyun(&mut connection, actor_user_id).await?;
        let mut query = users::table
            .inner_join(user_roles::table.inner_join(roles::table))
            .filter(users::user_deleted_at.is_null())
            .filter(users::user_hard_purged_at.is_null())
            .filter(users::user_is_system_actor.eq(false))
            .select((
                users::user_id,
                users::user_name,
                users::user_email,
                user_roles::role_id,
            ))
            .order(users::user_id.asc())
            .into_boxed();
        if let Some(after) = after {
            query = query.filter(users::user_id.gt(after));
        }
        if let Some(search) = search {
            let pattern = format!("{}%", escape_like(search.as_ref()));
            query = query.filter(
                users::user_name
                    .like(pattern.clone())
                    .escape('\\')
                    .or(users::user_email.like(pattern).escape('\\')),
            );
        }
        let rows = query
            .limit(i64::from(page_size.into_inner()) + 1)
            .load::<AuthorizationUserRow>(&mut connection)
            .await?;
        map_user_page(rows, page_size)
    }

    pub async fn authorization_roles(
        &self,
        actor_user_id: Uuid,
    ) -> Result<Vec<RoleDefinition>, AuthorizationError> {
        let mut connection = self.connection().await?;
        ensure_current_younghyun(&mut connection, actor_user_id).await?;
        let rows = roles::table
            .order(roles::role_name.asc())
            .limit(MAX_AUTHORIZATION_CATALOG_ROWS)
            .select((roles::role_id, roles::role_description))
            .load::<RoleDefinitionRow>(&mut connection)
            .await?;
        rows.into_iter()
            .map(|row| {
                let role_type = RoleType::from_uuid(row.role_id)
                    .ok_or(AuthorizationError::InvalidRoleId(row.role_id))?;
                Ok(RoleDefinition {
                    role_type,
                    description: row.role_description,
                })
            })
            .collect()
    }

    pub async fn authorization_permissions(
        &self,
        actor_user_id: Uuid,
    ) -> Result<Vec<PermissionDefinition>, AuthorizationError> {
        let mut connection = self.connection().await?;
        ensure_current_younghyun(&mut connection, actor_user_id).await?;
        let rows = permissions::table
            .order(permissions::permission_name.asc())
            .limit(MAX_AUTHORIZATION_CATALOG_ROWS)
            .select((
                permissions::permission_id,
                permissions::permission_name,
                permissions::permission_description,
            ))
            .load::<PermissionDefinitionRow>(&mut connection)
            .await?;
        rows.into_iter()
            .map(|row| {
                Ok(PermissionDefinition {
                    permission_id: row.permission_id,
                    permission_name: PermissionName::try_new(row.permission_name)
                        .map_err(|_| AuthorizationError::InvalidPermissionName)?,
                    description: row.permission_description,
                })
            })
            .collect()
    }

    pub async fn authorization_role_permissions(
        &self,
        actor_user_id: Uuid,
        after: Option<Uuid>,
        page_size: AuthorizationPageSize,
    ) -> Result<AuthorizationPage<RolePermissionBinding>, AuthorizationError> {
        let mut connection = self.connection().await?;
        ensure_current_younghyun(&mut connection, actor_user_id).await?;
        let mut query = role_permissions::table
            .inner_join(permissions::table)
            .select((
                role_permissions::role_permission_id,
                role_permissions::role_id,
                role_permissions::permission_id,
                permissions::permission_name,
            ))
            .order(role_permissions::role_permission_id.asc())
            .into_boxed();
        if let Some(after) = after {
            query = query.filter(role_permissions::role_permission_id.gt(after));
        }
        let mut rows = query
            .limit(i64::from(page_size.into_inner()) + 1)
            .load::<RolePermissionBindingRow>(&mut connection)
            .await?;
        let next_cursor = take_next_cursor(&mut rows, page_size, |row| row.role_permission_id);
        let items = rows
            .into_iter()
            .map(|row| {
                Ok(RolePermissionBinding {
                    role_permission_id: row.role_permission_id,
                    role_type: RoleType::from_uuid(row.role_id)
                        .ok_or(AuthorizationError::InvalidRoleId(row.role_id))?,
                    permission_id: row.permission_id,
                    permission_name: PermissionName::try_new(row.permission_name)
                        .map_err(|_| AuthorizationError::InvalidPermissionName)?,
                })
            })
            .collect::<Result<Vec<_>, AuthorizationError>>()?;
        Ok(AuthorizationPage { items, next_cursor })
    }

}

fn map_user_page(
    mut rows: Vec<AuthorizationUserRow>,
    page_size: AuthorizationPageSize,
) -> Result<AuthorizationPage<AuthorizationUser>, AuthorizationError> {
    let next_cursor = take_next_cursor(&mut rows, page_size, |row| row.user_id);
    let items = rows
        .into_iter()
        .map(|row| {
            Ok(AuthorizationUser {
                user_id: row.user_id,
                user_name: row.user_name,
                user_email: row.user_email,
                role_type: RoleType::from_uuid(row.role_id)
                    .ok_or(AuthorizationError::InvalidRoleId(row.role_id))?,
            })
        })
        .collect::<Result<Vec<_>, AuthorizationError>>()?;
    Ok(AuthorizationPage { items, next_cursor })
}

fn take_next_cursor<T>(
    rows: &mut Vec<T>,
    page_size: AuthorizationPageSize,
    id: impl Fn(&T) -> Uuid,
) -> Option<Uuid> {
    if rows.len() <= usize::from(page_size.into_inner()) {
        return None;
    }
    rows.pop();
    rows.last().map(id)
}

fn escape_like(value: &str) -> String {
    value.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")
}
