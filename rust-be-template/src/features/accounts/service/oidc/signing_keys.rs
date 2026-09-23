//! Provider signing keys with rate-limited refresh after rotation.
//!
//! Discovery loads the JWKS once at startup. Providers rotate keys, and a token signed by a
//! new key names a `kid` the cached set lacks; without a refresh every OIDC login would fail
//! until restart. Refreshes happen on such an unknown key and from an hourly job, but never
//! more than once per [`MIN_REFRESH_INTERVAL`], so forged `kid` values cannot turn each
//! callback into an outbound request.

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use openidconnect::{
    ClientId, ClientSecret, EndpointMaybeSet, EndpointNotSet, EndpointSet, JsonWebKeySet,
    RedirectUrl,
    core::{CoreClient, CoreProviderMetadata},
};

use super::http_client::OidcHttpClient;
use crate::features::accounts::error::AccountError;

pub(super) const MIN_REFRESH_INTERVAL: Duration = Duration::from_secs(5 * 60);

pub(super) type DiscoveredCoreClient = CoreClient<
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;

/// Outcome of a refresh request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum KeyRefresh {
    Installed {
        keys: usize,
    },
    /// A refresh ran too recently; the current keys stay in use.
    RateLimited,
}

/// Rebuildable client whose JWKS can be replaced without restarting the process.
pub(super) struct SigningKeys {
    metadata: CoreProviderMetadata,
    client_id: ClientId,
    client_secret: Option<ClientSecret>,
    redirect_url: RedirectUrl,
    client: tokio::sync::RwLock<Arc<DiscoveredCoreClient>>,
    /// Serializes refreshes and records the last attempt, successful or not.
    last_attempt: tokio::sync::Mutex<Option<Instant>>,
}

impl SigningKeys {
    pub(super) fn new(
        metadata: CoreProviderMetadata,
        client_id: ClientId,
        client_secret: Option<ClientSecret>,
        redirect_url: RedirectUrl,
    ) -> Self {
        let client = build_client(&metadata, &client_id, &client_secret, &redirect_url);
        Self {
            metadata,
            client_id,
            client_secret,
            redirect_url,
            client: tokio::sync::RwLock::new(Arc::new(client)),
            last_attempt: tokio::sync::Mutex::new(Some(Instant::now())),
        }
    }

    /// The client carrying the newest installed key set.
    pub(super) async fn client(&self) -> Arc<DiscoveredCoreClient> {
        Arc::clone(&*self.client.read().await)
    }

    /// Refetches the JWKS unless a refresh ran within [`MIN_REFRESH_INTERVAL`].
    ///
    /// An empty or unreadable key set leaves the current keys installed.
    pub(super) async fn refresh(
        &self,
        http_client: &OidcHttpClient,
    ) -> Result<KeyRefresh, AccountError> {
        let mut last_attempt = self.last_attempt.lock().await;
        let now = Instant::now();
        if !refresh_is_due(*last_attempt, now) {
            return Ok(KeyRefresh::RateLimited);
        }
        *last_attempt = Some(now);
        let jwks = JsonWebKeySet::fetch_async(self.metadata.jwks_uri(), http_client)
            .await
            .map_err(|error| AccountError::OidcTokenExchange(anyhow::Error::new(error)))?;
        let keys = jwks.keys().len();
        if keys == 0 {
            return Err(AccountError::OidcTokenExchange(anyhow::anyhow!(
                "provider returned an empty signing key set"
            )));
        }
        let metadata = self.metadata.clone().set_jwks(jwks);
        let client = build_client(
            &metadata,
            &self.client_id,
            &self.client_secret,
            &self.redirect_url,
        );
        *self.client.write().await = Arc::new(client);
        drop(last_attempt);
        tracing::info!(
            event = "oidc_signing_keys_refreshed",
            keys,
            "Refreshed OpenID Connect signing keys"
        );
        Ok(KeyRefresh::Installed { keys })
    }
}

fn refresh_is_due(last_attempt: Option<Instant>, now: Instant) -> bool {
    match last_attempt {
        Some(last_attempt) => now.saturating_duration_since(last_attempt) >= MIN_REFRESH_INTERVAL,
        None => true,
    }
}

fn build_client(
    metadata: &CoreProviderMetadata,
    client_id: &ClientId,
    client_secret: &Option<ClientSecret>,
    redirect_url: &RedirectUrl,
) -> DiscoveredCoreClient {
    CoreClient::from_provider_metadata(metadata.clone(), client_id.clone(), client_secret.clone())
        .set_redirect_uri(redirect_url.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refreshes_are_spaced_by_the_minimum_interval() {
        let start = Instant::now();
        assert!(refresh_is_due(None, start));
        assert!(!refresh_is_due(
            Some(start),
            start + Duration::from_secs(299)
        ));
        assert!(refresh_is_due(Some(start), start + MIN_REFRESH_INTERVAL));
    }
}
