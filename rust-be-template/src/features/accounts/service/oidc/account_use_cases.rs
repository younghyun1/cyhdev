//! Local-account use cases for verified OpenID Connect identities.

use uuid::Uuid;

use crate::{
    features::accounts::{
        domain::{
            account::SessionPrincipal,
            oidc::{OidcIdentityClaims, OidcSessionReceipt},
            role::RoleType,
        },
        error::AccountError,
        service::{
            account_service::AccountService,
            authentication::password_within_auth_bound,
        },
    },
    util::crypto::verify_pw::verify_pw,
};

impl AccountService {
    pub async fn oidc_is_linked(&self, user_id: Uuid, issuer: &str) -> Result<bool, AccountError> {
        self.repository.oidc_is_linked(user_id, issuer).await
    }

    /// Creates a session only after issuer and subject resolve to an existing local link.
    pub async fn oidc_login(
        &self,
        identity: &OidcIdentityClaims,
        previous_session_token: Option<&str>,
    ) -> Result<OidcSessionReceipt, AccountError> {
        let _session_consistency = self.session_consistency.read().await;
        let account = self
            .repository
            .oidc_account_for_login(identity)
            .await?
            .ok_or(AccountError::OidcIdentityNotLinked)?;
        let principal: SessionPrincipal = account.into();
        let role = self
            .repository
            .role_for_user_or_insert_default(principal.user_id, RoleType::User)
            .await?;
        let session_token = self
            .sessions
            .create_principal(&principal, role, previous_session_token, None)
            .await?;
        Ok(OidcSessionReceipt {
            user_id: principal.user_id,
            session_token,
        })
    }

    /// Links a completed provider flow to the same verified local account that started it.
    pub async fn complete_oidc_link(
        &self,
        current_user_id: Uuid,
        expected_user_id: Uuid,
        identity: &OidcIdentityClaims,
        previous_session_token: Option<&str>,
    ) -> Result<OidcSessionReceipt, AccountError> {
        if current_user_id != expected_user_id {
            return Err(AccountError::OidcLinkSessionMismatch);
        }
        let _session_consistency = self.session_consistency.write().await;
        let principal = self
            .repository
            .link_oidc_identity(current_user_id, identity)
            .await?;
        let role = self
            .repository
            .role_for_user_or_insert_default(current_user_id, RoleType::User)
            .await?;
        let session_token = self
            .sessions
            .create_principal(&principal, role, previous_session_token, None)
            .await?;
        Ok(OidcSessionReceipt {
            user_id: current_user_id,
            session_token,
        })
    }

    /// Confirms the retained local password, removes the provider link, and rotates the session.
    pub async fn unlink_oidc(
        &self,
        user_id: Uuid,
        issuer: &str,
        current_password: &str,
        previous_session_token: Option<&str>,
    ) -> Result<OidcSessionReceipt, AccountError> {
        if !password_within_auth_bound(current_password) {
            return Err(AccountError::InvalidPassword);
        }
        let session_consistency_read = self.session_consistency.read().await;
        let candidate = self
            .repository
            .oidc_unlink_candidate(user_id, issuer)
            .await?;
        let password_job = self.try_password_job()?;
        let password_matches = verify_pw(current_password, &candidate.password_hash)
            .await
            .map_err(AccountError::PasswordVerification)?;
        drop(password_job);
        if !password_matches {
            return Err(AccountError::WrongPassword);
        }
        drop(session_consistency_read);

        let _session_consistency = self.session_consistency.write().await;
        let principal = self
            .repository
            .unlink_oidc_identity(user_id, issuer, &candidate.password_hash)
            .await?;
        let role = self
            .repository
            .role_for_user_or_insert_default(user_id, RoleType::User)
            .await?;
        let session_token = self
            .sessions
            .create_principal(&principal, role, previous_session_token, None)
            .await?;
        Ok(OidcSessionReceipt {
            user_id,
            session_token,
        })
    }
}
