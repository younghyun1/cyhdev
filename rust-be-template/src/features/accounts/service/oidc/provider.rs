//! Provider-neutral OpenID Connect Authorization Code flow with PKCE.

use std::sync::Arc;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use openidconnect::{
    AuthorizationCode, CsrfToken, EndpointMaybeSet, EndpointNotSet, EndpointSet, Nonce,
    PkceCodeChallenge, PkceCodeVerifier, Scope, TokenResponse,
    core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata},
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
    flow_store::{CompletedLink, OidcFlowStores, OneTimeToken, PendingAuthorization},
    http_client::OidcHttpClient,
    validation::{identity_from_claims, verify_access_token_hash},
};

const OIDC_SECRET_BYTES: usize = 32;
const MAX_AUTHORIZATION_CODE_BYTES: usize = 8 * 1024;

type DiscoveredCoreClient = CoreClient<
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;

pub struct OidcService {
    enabled: Option<EnabledOidcService>,
}

struct EnabledOidcService {
    provider_name: Arc<str>,
    issuer: Arc<str>,
    client: DiscoveredCoreClient,
    http_client: OidcHttpClient,
    flows: OidcFlowStores,
}

pub(crate) enum OidcCallbackOutcome {
    Login(OidcIdentityClaims),
    LinkReady { completion_token: OneTimeToken },
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
        let client = CoreClient::from_provider_metadata(
            provider_metadata,
            config.client_id,
            config.client_secret,
        )
        .set_redirect_uri(config.redirect_url);

        Ok(Self {
            enabled: Some(EnabledOidcService {
                provider_name: config.provider_name,
                issuer,
                client,
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

    pub(crate) async fn start_authorization(
        &self,
        mode: OidcFlowMode,
    ) -> Result<String, AccountError> {
        let enabled = self.enabled.as_ref().ok_or(AccountError::OidcDisabled)?;
        let pkce_secret = random_secret()?;
        let nonce = random_secret()?;
        let verifier = PkceCodeVerifier::new(pkce_secret.as_str().to_owned());
        let challenge = PkceCodeChallenge::from_code_verifier_sha256(&verifier);
        let state = enabled
            .flows
            .insert_pending(PendingAuthorization {
                mode,
                pkce_verifier: pkce_secret,
                nonce: nonce.clone(),
            })
            .await?;
        let (authorization_url, _, _) = enabled
            .client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                move || CsrfToken::new(state.expose().to_owned()),
                move || Nonce::new(nonce.as_str().to_owned()),
            )
            .add_scope(Scope::new("email".to_owned()))
            .set_pkce_challenge(challenge)
            .url();
        Ok(authorization_url.to_string())
    }

    pub(crate) async fn cancel_authorization(&self, state: &str) -> Option<OidcFlowMode> {
        let enabled = self.enabled.as_ref()?;
        enabled
            .flows
            .take_pending(state)
            .await
            .map(|pending| pending.mode)
    }

    pub(crate) async fn finish_authorization(
        &self,
        state: &str,
        code: &str,
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
        let token_response = enabled
            .client
            .exchange_code(AuthorizationCode::new(code.to_owned()))
            .map_err(|error| AccountError::OidcTokenExchange(anyhow::Error::new(error)))?
            .set_pkce_verifier(PkceCodeVerifier::new(
                pending.pkce_verifier.as_str().to_owned(),
            ))
            .request_async(&enabled.http_client)
            .await
            .map_err(|error| AccountError::OidcTokenExchange(anyhow::Error::new(error)))?;
        let id_token = token_response.id_token().ok_or_else(|| {
            AccountError::OidcTokenValidation(anyhow::anyhow!("missing ID token"))
        })?;
        let verifier = enabled
            .client
            .id_token_verifier()
            .require_issuer_match(true)
            .require_audience_match(true);
        let claims = id_token
            .claims(&verifier, &Nonce::new(pending.nonce.as_str().to_owned()))
            .map_err(|error| AccountError::OidcTokenValidation(anyhow::Error::new(error)))?;
        if claims.issuer().as_str() != enabled.issuer.as_ref()
            || !claims
                .audiences()
                .iter()
                .any(|audience| audience.as_str() == enabled.client.client_id().as_str())
        {
            return Err(AccountError::OidcTokenValidation(anyhow::anyhow!(
                "issuer or audience mismatch"
            )));
        }
        verify_access_token_hash(id_token, claims, &token_response, &verifier)?;
        let identity = identity_from_claims(claims)?;

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
}

fn random_secret() -> Result<Zeroizing<String>, AccountError> {
    let mut bytes = Zeroizing::new([0_u8; OIDC_SECRET_BYTES]);
    getrandom::fill(bytes.as_mut()).map_err(AccountError::OidcFlowEntropy)?;
    Ok(Zeroizing::new(URL_SAFE_NO_PAD.encode(bytes.as_ref())))
}
