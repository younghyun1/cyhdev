//! Provider-neutral OpenID Connect Authorization Code flow with PKCE.

use std::sync::Arc;

use openidconnect::{
    AuthorizationCode, CsrfToken, Nonce, PkceCodeChallenge, PkceCodeVerifier, Scope,
    core::{CoreAuthenticationFlow, CoreProviderMetadata},
};
use zeroize::Zeroizing;

use crate::{
    features::accounts::{
        domain::oidc::{OidcFlowMode, OidcIdentityClaims},
        error::AccountError,
    },
    init::state::{DeploymentEnvironment, PublicAppOrigin},
};

use super::{
    config::OidcConfig,
    flow_store::{
        BrowserBinding, CompletedLink, OidcFlowStores, OneTimeToken, PendingAuthorization,
        random_token,
    },
    http_client::OidcHttpClient,
    signing_keys::{KeyRefresh, SigningKeys},
    validation::{IdentityRejection, validated_identity},
};

const MAX_AUTHORIZATION_CODE_BYTES: usize = 8 * 1024;

pub struct OidcService {
    enabled: Option<EnabledOidcService>,
}

struct EnabledOidcService {
    provider_name: Arc<str>,
    issuer: Arc<str>,
    keys: SigningKeys,
    http_client: OidcHttpClient,
    flows: OidcFlowStores,
}

pub(crate) enum OidcCallbackOutcome {
    Login(OidcIdentityClaims),
    LinkReady { completion_token: OneTimeToken },
}

/// A started flow: the provider URL and the secret for the browser-binding cookie.
pub(crate) struct OidcAuthorizationStart {
    pub(crate) authorization_url: String,
    pub(crate) browser_binding: OneTimeToken,
}

impl OidcService {
    pub(crate) async fn from_environment(
        deployment: DeploymentEnvironment,
        public_origin: &PublicAppOrigin,
    ) -> anyhow::Result<Self> {
        let config = match OidcConfig::from_environment(deployment, public_origin)? {
            Some(config) => config,
            None => return Ok(Self { enabled: None }),
        };
        let http_client = OidcHttpClient::new(config.allow_loopback_http)?;
        let provider_metadata =
            CoreProviderMetadata::discover_async(config.issuer.clone(), &http_client)
                .await
                .map_err(|error| anyhow::anyhow!("OIDC discovery failed: {error}"))?;
        if provider_metadata.token_endpoint().is_none() {
            return Err(anyhow::anyhow!(
                "OIDC provider metadata omitted the token endpoint required by Authorization Code flow"
            ));
        }
        let issuer: Arc<str> = Arc::from(provider_metadata.issuer().as_str());
        let keys = SigningKeys::new(
            provider_metadata,
            config.client_id,
            config.client_secret,
            config.redirect_url,
        );

        Ok(Self {
            enabled: Some(EnabledOidcService {
                provider_name: config.provider_name,
                issuer,
                keys,
                http_client,
                flows: OidcFlowStores::default(),
            }),
        })
    }

    pub(crate) fn is_enabled(&self) -> bool {
        self.enabled.is_some()
    }

    pub(crate) fn provider_name(&self) -> Option<&str> {
        self.enabled
            .as_ref()
            .map(|enabled| enabled.provider_name.as_ref())
    }

    pub(crate) fn issuer(&self) -> Option<&str> {
        self.enabled.as_ref().map(|enabled| enabled.issuer.as_ref())
    }

    /// Starts a flow bound to the browser that will present the returned binding cookie.
    pub(crate) async fn start_authorization(
        &self,
        mode: OidcFlowMode,
    ) -> Result<OidcAuthorizationStart, AccountError> {
        let enabled = self.enabled.as_ref().ok_or(AccountError::OidcDisabled)?;
        let pkce_secret = random_secret()?;
        let nonce = random_secret()?;
        let browser_binding = random_token().map_err(AccountError::OidcFlowEntropy)?;
        let verifier = PkceCodeVerifier::new(pkce_secret.as_str().to_owned());
        let challenge = PkceCodeChallenge::from_code_verifier_sha256(&verifier);
        let state = enabled
            .flows
            .insert_pending(PendingAuthorization {
                mode,
                pkce_verifier: pkce_secret,
                nonce: nonce.clone(),
                browser_binding: BrowserBinding::of_cookie(browser_binding.expose()),
            })
            .await?;
        let client = enabled.keys.client().await;
        let (authorization_url, _, _) = client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                move || CsrfToken::new(state.expose().to_owned()),
                move || Nonce::new(nonce.as_str().to_owned()),
            )
            .add_scope(Scope::new("email".to_owned()))
            .set_pkce_challenge(challenge)
            .url();
        Ok(OidcAuthorizationStart {
            authorization_url: authorization_url.to_string(),
            browser_binding,
        })
    }

    /// Consumes a denied flow; its mode is revealed only to the browser that started it.
    pub(crate) async fn cancel_authorization(
        &self,
        state: &str,
        browser_binding: Option<&str>,
    ) -> Option<OidcFlowMode> {
        let enabled = self.enabled.as_ref()?;
        let pending = enabled.flows.take_pending(state).await?;
        binding_matches(&pending, browser_binding).then_some(pending.mode)
    }

    pub(crate) async fn finish_authorization(
        &self,
        state: &str,
        code: &str,
        browser_binding: Option<&str>,
    ) -> Result<OidcCallbackOutcome, AccountError> {
        let enabled = self.enabled.as_ref().ok_or(AccountError::OidcDisabled)?;
        if code.is_empty() || code.len() > MAX_AUTHORIZATION_CODE_BYTES {
            return Err(AccountError::OidcFlowRejected);
        }
        let pending = enabled
            .flows
            .take_pending(state)
            .await
            .ok_or(AccountError::OidcFlowRejected)?;
        // A callback URL replayed in another browser lacks the cookie, so an attacker cannot
        // sign a victim into the attacker's account by making them open it.
        if !binding_matches(&pending, browser_binding) {
            return Err(AccountError::OidcFlowRejected);
        }
        let client = enabled.keys.client().await;
        let token_response = client
            .exchange_code(AuthorizationCode::new(code.to_owned()))
            .map_err(|error| AccountError::OidcTokenExchange(anyhow::Error::new(error)))?
            .set_pkce_verifier(PkceCodeVerifier::new(
                pending.pkce_verifier.as_str().to_owned(),
            ))
            .request_async(&enabled.http_client)
            .await
            .map_err(|error| AccountError::OidcTokenExchange(anyhow::Error::new(error)))?;
        let identity =
            match validated_identity(&client, &token_response, &pending.nonce, &enabled.issuer) {
                Ok(identity) => identity,
                Err(IdentityRejection::Invalid(error)) => return Err(error),
                Err(IdentityRejection::UnknownSigningKey) => {
                    self.identity_after_key_refresh(enabled, &token_response, &pending.nonce)
                        .await?
                }
            };

        match pending.mode {
            OidcFlowMode::Login => Ok(OidcCallbackOutcome::Login(identity)),
            OidcFlowMode::Link { expected_user_id } => {
                let completion_token = enabled
                    .flows
                    .insert_completed_link(CompletedLink {
                        expected_user_id,
                        identity,
                    })
                    .await?;
                Ok(OidcCallbackOutcome::LinkReady { completion_token })
            }
        }
    }

    pub(crate) async fn consume_link_completion(
        &self,
        token: &str,
    ) -> Result<(uuid::Uuid, OidcIdentityClaims), AccountError> {
        let enabled = self.enabled.as_ref().ok_or(AccountError::OidcDisabled)?;
        let completed = enabled
            .flows
            .take_completed_link(token)
            .await
            .ok_or(AccountError::OidcFlowRejected)?;
        Ok((completed.expected_user_id, completed.identity))
    }

    /// Periodic refresh so rotated keys are installed before the first token needs them.
    pub(crate) async fn refresh_signing_keys(&self) {
        let Some(enabled) = self.enabled.as_ref() else {
            return;
        };
        if let Err(error) = enabled.keys.refresh(&enabled.http_client).await {
            tracing::warn!(
                event = "oidc_signing_key_refresh_failed",
                error = %error,
                "OpenID Connect signing key refresh failed; keeping current keys"
            );
        }
    }

    async fn identity_after_key_refresh(
        &self,
        enabled: &EnabledOidcService,
        token_response: &openidconnect::core::CoreTokenResponse,
        nonce: &str,
    ) -> Result<OidcIdentityClaims, AccountError> {
        let unknown_key = || {
            AccountError::OidcTokenValidation(anyhow::anyhow!(
                "ID token signing key is not in the provider key set"
            ))
        };
        match enabled.keys.refresh(&enabled.http_client).await? {
            KeyRefresh::Installed { .. } => {}
            KeyRefresh::RateLimited => return Err(unknown_key()),
        }
        let client = enabled.keys.client().await;
        match validated_identity(&client, token_response, nonce, &enabled.issuer) {
            Ok(identity) => Ok(identity),
            Err(IdentityRejection::Invalid(error)) => Err(error),
            Err(IdentityRejection::UnknownSigningKey) => Err(unknown_key()),
        }
    }
}

fn binding_matches(pending: &PendingAuthorization, browser_binding: Option<&str>) -> bool {
    browser_binding.is_some_and(|value| BrowserBinding::of_cookie(value) == pending.browser_binding)
}

fn random_secret() -> Result<Zeroizing<String>, AccountError> {
    let token = random_token().map_err(AccountError::OidcFlowEntropy)?;
    Ok(Zeroizing::new(token.expose().to_owned()))
}
