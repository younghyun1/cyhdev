//! Audited authorization administration use cases.

use uuid::Uuid;

use crate::features::accounts::{
    authorization_error::AuthorizationError,
    domain::{
        authorization::{
            AuthorizationAuditCursor, AuthorizationAuditPage, AuthorizationPage,
            AuthorizationPageSize, AuthorizationReason, AuthorizationSearch, AuthorizationUser,
            DEFAULT_AUTHORIZATION_PAGE_SIZE, PermissionDefinition, PermissionName,
            RoleAssignmentReceipt, RoleDefinition, RolePermissionBinding, RolePermissionReceipt,
        },
        role::RoleType,
    },
    service::account_service::AccountService,
};

impl AccountService {
    pub async fn authorization_users(
        &self,
        actor_user_id: Uuid,
        search: Option<String>,
        after: Option<Uuid>,
        limit: Option<u16>,
    ) -> Result<AuthorizationPage<AuthorizationUser>, AuthorizationError> {
        let search = validated_search(search)?;
        self.repository
            .authorization_users(
                actor_user_id,
                search.as_ref(),
                after,
                validated_page_size(limit)?,
            )
            .await
    }

    pub async fn authorization_roles(
        &self,
        actor_user_id: Uuid,
    ) -> Result<Vec<RoleDefinition>, AuthorizationError> {
        self.repository.authorization_roles(actor_user_id).await
    }

    pub async fn authorization_permissions(
        &self,
        actor_user_id: Uuid,
    ) -> Result<Vec<PermissionDefinition>, AuthorizationError> {
        self.repository
            .authorization_permissions(actor_user_id)
            .await
    }

    pub async fn authorization_role_permissions(
        &self,
        actor_user_id: Uuid,
        after: Option<Uuid>,
        limit: Option<u16>,
    ) -> Result<AuthorizationPage<RolePermissionBinding>, AuthorizationError> {
        self.repository
            .authorization_role_permissions(actor_user_id, after, validated_page_size(limit)?)
            .await
    }

    pub async fn authorization_audit_events(
        &self,
        actor_user_id: Uuid,
        before: Option<AuthorizationAuditCursor>,
        limit: Option<u16>,
    ) -> Result<AuthorizationAuditPage, AuthorizationError> {
        self.repository
            .authorization_audit_events(actor_user_id, before, validated_page_size(limit)?)
            .await
    }

    pub async fn assign_role_as_administrator(
        &self,
        actor_user_id: Uuid,
        target_user_id: Uuid,
        role_id: Uuid,
        reason: String,
        request_id: Option<Uuid>,
    ) -> Result<RoleAssignmentReceipt, AuthorizationError> {
        let role_type =
            RoleType::from_uuid(role_id).ok_or(AuthorizationError::InvalidRoleId(role_id))?;
        let reason =
            AuthorizationReason::try_new(reason).map_err(|_| AuthorizationError::InvalidReason)?;
        let _session_consistency = self.session_consistency.write().await;
        let receipt = self
            .repository
            .assign_role_with_audit(
                actor_user_id,
                target_user_id,
                role_type,
                &reason,
                request_id,
            )
            .await?;
        self.refresh_sessions_after_commit(target_user_id, "admin_assign_role")
            .await;
        Ok(receipt)
    }

    pub async fn set_role_permission_as_administrator(
        &self,
        actor_user_id: Uuid,
        role_id: Uuid,
        permission_id: Uuid,
        enabled: bool,
        reason: String,
        request_id: Option<Uuid>,
    ) -> Result<RolePermissionReceipt, AuthorizationError> {
        let role_type =
            RoleType::from_uuid(role_id).ok_or(AuthorizationError::InvalidRoleId(role_id))?;
        let reason =
            AuthorizationReason::try_new(reason).map_err(|_| AuthorizationError::InvalidReason)?;
        let _session_consistency = self.session_consistency.write().await;
        self.repository
            .set_role_permission_with_audit(
                actor_user_id,
                role_type,
                permission_id,
                enabled,
                &reason,
                request_id,
            )
            .await
    }

    /// Reads current PostgreSQL authority. Permission decisions are never cached.
    pub async fn has_current_permission(
        &self,
        user_id: Uuid,
        permission_name: &str,
    ) -> Result<bool, AuthorizationError> {
        let permission_name = PermissionName::try_new(permission_name.to_owned())
            .map_err(|_| AuthorizationError::InvalidPermissionName)?;
        self.repository
            .has_current_permission(user_id, &permission_name)
            .await
    }
}

fn validated_search(
    search: Option<String>,
) -> Result<Option<AuthorizationSearch>, AuthorizationError> {
    match search {
        Some(search) if search.trim().is_empty() => Ok(None),
        Some(search) => AuthorizationSearch::try_new(search)
            .map(Some)
            .map_err(|_| AuthorizationError::InvalidSearch),
        None => Ok(None),
    }
}

fn validated_page_size(limit: Option<u16>) -> Result<AuthorizationPageSize, AuthorizationError> {
    AuthorizationPageSize::try_new(limit.unwrap_or(DEFAULT_AUTHORIZATION_PAGE_SIZE))
        .map_err(|_| AuthorizationError::InvalidPageSize)
}
