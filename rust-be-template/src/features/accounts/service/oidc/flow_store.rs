//! Fixed-capacity one-time state for single-process OIDC authorization flows.

use std::time::Duration;

use sha2::{Digest, Sha256};
use uuid::Uuid;
use zeroize::Zeroizing;

use super::one_time_store::OneTimeStore;
pub(crate) use super::one_time_store::OneTimeToken;
pub(super) use super::one_time_store::random_token;
use crate::features::accounts::{
    domain::oidc::{OidcFlowMode, OidcIdentityClaims},
    error::AccountError,
};

/// Pending login flows; the largest class because anyone may start one.
pub const MAX_PENDING_OIDC_LOGINS: usize = 512;
/// Pending link flows; starting one needs a password-confirmed verified session.
pub const MAX_PENDING_OIDC_LINKS: usize = 128;
/// Validated link identities waiting for same-origin completion.
pub const MAX_COMPLETED_OIDC_LINKS: usize = 128;
const OIDC_FLOW_TTL: Duration = Duration::from_secs(10 * 60);
const OIDC_LINK_COMPLETION_TTL: Duration = Duration::from_secs(5 * 60);

/// SHA-256 of the browser-binding cookie that started a flow.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) struct BrowserBinding([u8; 32]);

impl BrowserBinding {
    pub(super) fn of_cookie(value: &str) -> Self {
        Self(Sha256::digest(value.as_bytes()).into())
    }
}

pub(super) struct PendingAuthorization {
    pub(super) mode: OidcFlowMode,
    pub(super) pkce_verifier: Zeroizing<String>,
    pub(super) nonce: Zeroizing<String>,
    pub(super) browser_binding: BrowserBinding,
}

pub(super) struct CompletedLink {
    pub(super) expected_user_id: Uuid,
    pub(super) identity: OidcIdentityClaims,
}

pub(super) struct OidcFlowStores {
    pending_logins: OneTimeStore<PendingAuthorization>,
    pending_links: OneTimeStore<PendingAuthorization>,
    completed_links: OneTimeStore<CompletedLink>,
}

impl Default for OidcFlowStores {
    fn default() -> Self {
        Self {
            pending_logins: OneTimeStore::new(MAX_PENDING_OIDC_LOGINS, OIDC_FLOW_TTL),
            pending_links: OneTimeStore::new(MAX_PENDING_OIDC_LINKS, OIDC_FLOW_TTL),
            completed_links: OneTimeStore::new(MAX_COMPLETED_OIDC_LINKS, OIDC_LINK_COMPLETION_TTL),
        }
    }
}

impl OidcFlowStores {
    pub(super) async fn insert_pending(
        &self,
        flow: PendingAuthorization,
    ) -> Result<OneTimeToken, AccountError> {
        match flow.mode {
            OidcFlowMode::Login => self.pending_logins.insert(flow).await,
            OidcFlowMode::Link { .. } => self.pending_links.insert(flow).await,
        }
    }

    /// Consumes pending state; the state token does not reveal its class, so both are tried.
    pub(super) async fn take_pending(&self, token: &str) -> Option<PendingAuthorization> {
        match self.pending_logins.take(token).await {
            Some(flow) => Some(flow),
            None => self.pending_links.take(token).await,
        }
    }

    pub(super) async fn insert_completed_link(
        &self,
        link: CompletedLink,
    ) -> Result<OneTimeToken, AccountError> {
        self.completed_links.insert(link).await
    }

    pub(super) async fn take_completed_link(&self, token: &str) -> Option<CompletedLink> {
        self.completed_links.take(token).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn login_floods_cannot_evict_pending_links() -> Result<(), AccountError> {
        let stores = OidcFlowStores::default();
        let link = stores
            .insert_pending(pending(OidcFlowMode::Link {
                expected_user_id: Uuid::nil(),
            }))
            .await?;
        for _ in 0..=MAX_PENDING_OIDC_LOGINS {
            let _ = stores.insert_pending(pending(OidcFlowMode::Login)).await?;
        }
        let taken = stores.take_pending(link.expose()).await;
        assert!(taken.is_some_and(|flow| matches!(flow.mode, OidcFlowMode::Link { .. })));
        Ok(())
    }

    #[test]
    fn binding_digests_distinguish_cookie_values() {
        assert!(BrowserBinding::of_cookie("a") == BrowserBinding::of_cookie("a"));
        assert!(BrowserBinding::of_cookie("a") != BrowserBinding::of_cookie("b"));
    }

    fn pending(mode: OidcFlowMode) -> PendingAuthorization {
        PendingAuthorization {
            mode,
            pkce_verifier: Zeroizing::new(String::new()),
            nonce: Zeroizing::new(String::new()),
            browser_binding: BrowserBinding::of_cookie("test"),
        }
    }
}
