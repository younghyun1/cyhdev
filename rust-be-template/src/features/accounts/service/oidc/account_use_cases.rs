//! Local-account use cases for verified OpenID Connect identities.

use chrono::Utc;
use lettre::AsyncTransport;
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
            account_service::{AccountService, MAX_EMAIL_JOBS},
            authentication::password_within_auth_bound,
            password_work::PasswordBudget,
        },
    },
    util::email::sign_in_method_linked::SignInMethodLinkedEmail,
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

    /// Re-proves the local password before a provider link may start.
    ///
    /// A link grants password-free sign-in, so a session alone, which may be a stolen
    /// cookie, is not enough. Accounts without a local password cannot exist through signup;
    /// if one ever does, there is no stronger re-authentication to offer, so it is refused.
    pub async fn confirm_oidc_link_start(
        &self,
        user_id: Uuid,
        current_password: &str,
    ) -> Result<(), AccountError> {
        if !password_within_auth_bound(current_password) {
            return Err(AccountError::InvalidPassword);
        }
        let candidate = self.repository.account_deletion_candidate(user_id).await?;
        if candidate.password_hash.is_empty() {
            return Err(AccountError::OidcAnotherLoginRequired);
        }
        let password_matches = self
            .verify_password(
                PasswordBudget::Confirmation,
                current_password,
                &candidate.password_hash,
            )
            .await?;
        if password_matches {
            Ok(())
        } else {
            Err(AccountError::WrongPassword)
        }
    }

    /// Links a completed provider flow to the same verified local account that started it.
    pub async fn complete_oidc_link(
        &self,
        current_user_id: Uuid,
        expected_user_id: Uuid,
        identity: &OidcIdentityClaims,
        provider_name: &str,
        previous_session_token: Option<&str>,
    ) -> Result<OidcSessionReceipt, AccountError> {
        if current_user_id != expected_user_id {
            return Err(AccountError::OidcLinkSessionMismatch);
        }
        let _session_consistency = self.session_consistency.write().await;
        let linked = self
            .repository
            .link_oidc_identity(current_user_id, identity)
            .await?;
        if linked.newly_linked {
            self.send_sign_in_method_linked_email(linked.owner_email, provider_name);
        }
        let principal = linked.principal;
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
        let password_matches = self
            .verify_password(
                PasswordBudget::Confirmation,
                current_password,
                &candidate.password_hash,
            )
            .await?;
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

    /// Tells the owner a sign-in method was added, through the bounded email path.
    fn send_sign_in_method_linked_email(&self, owner_email: String, provider_name: &str) {
        let email_job = match self.email_jobs.clone().try_acquire_owned() {
            Ok(email_job) => email_job,
            Err(_) => {
                tracing::warn!(
                    event = "auth_email_work_rejected",
                    email_kind = "sign_in_method_linked",
                    max_jobs = MAX_EMAIL_JOBS,
                    "Authentication email work rejected"
                );
                return;
            }
        };
        let email_client = self.email_client.clone();
        let email = SignInMethodLinkedEmail::new(provider_name, Utc::now());
        tokio::spawn(async move {
            let _email_job = email_job;
            let message = match email.to_message(&owner_email) {
                Ok(message) => message,
                Err(error) => {
                    tracing::error!(%error, "Could not build sign-in method notice");
                    return;
                }
            };
            if let Err(error) = email_client.send(message).await {
                tracing::error!(%error, "Could not send sign-in method notice");
            }
        });
    }
}
