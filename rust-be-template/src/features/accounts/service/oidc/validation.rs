//! ID-token signature, claim, and access-token-hash validation with bounded extraction.

use openidconnect::{
    AccessTokenHash, ClaimsVerificationError, Nonce, OAuth2TokenResponse,
    SignatureVerificationError, TokenResponse,
};

use super::signing_keys::DiscoveredCoreClient;
use crate::features::accounts::{
    domain::oidc::{
        MAX_OIDC_ISSUER_BYTES, MAX_OIDC_PROVIDER_EMAIL_BYTES, MAX_OIDC_SUBJECT_BYTES,
        OidcIdentityClaims,
    },
    error::AccountError,
};

/// Why an ID token was not accepted.
pub(super) enum IdentityRejection {
    /// The token names a signing key the cached JWKS lacks; a key refresh may fix it.
    UnknownSigningKey,
    Invalid(AccountError),
}

impl From<AccountError> for IdentityRejection {
    fn from(error: AccountError) -> Self {
        Self::Invalid(error)
    }
}

/// Validates the ID token against one key set and returns the owned identity claims.
pub(super) fn validated_identity(
    client: &DiscoveredCoreClient,
    token_response: &openidconnect::core::CoreTokenResponse,
    nonce: &str,
    expected_issuer: &str,
) -> Result<OidcIdentityClaims, IdentityRejection> {
    let id_token = token_response
        .id_token()
        .ok_or_else(|| AccountError::OidcTokenValidation(anyhow::anyhow!("missing ID token")))?;
    let verifier = client
        .id_token_verifier()
        .require_issuer_match(true)
        .require_audience_match(true);
    let claims = match id_token.claims(&verifier, &Nonce::new(nonce.to_owned())) {
        Ok(claims) => claims,
        Err(ClaimsVerificationError::SignatureVerification(
            SignatureVerificationError::NoMatchingKey,
        )) => return Err(IdentityRejection::UnknownSigningKey),
        Err(error) => {
            return Err(AccountError::OidcTokenValidation(anyhow::Error::new(error)).into());
        }
    };
    if claims.issuer().as_str() != expected_issuer
        || !claims
            .audiences()
            .iter()
            .any(|audience| audience.as_str() == client.client_id().as_str())
    {
        return Err(AccountError::OidcTokenValidation(anyhow::anyhow!(
            "issuer or audience mismatch"
        ))
        .into());
    }
    verify_access_token_hash(id_token, claims, token_response, &verifier)?;
    Ok(identity_from_claims(claims)?)
}

fn identity_from_claims(
    claims: &openidconnect::core::CoreIdTokenClaims,
) -> Result<OidcIdentityClaims, AccountError> {
    let issuer = claims.issuer().as_str();
    let subject = claims.subject().as_str();
    let provider_email = claims
        .email()
        .map(|email| email.as_str().trim())
        .filter(|email| !email.is_empty());
    let provider_email = match (provider_email, claims.email_verified()) {
        (Some(email), Some(true))
            if email.len() <= MAX_OIDC_PROVIDER_EMAIL_BYTES
                && email_address::EmailAddress::is_valid(email) =>
        {
            email
        }
        _ => return Err(AccountError::OidcProviderEmailRejected),
    };
    if issuer.is_empty()
        || issuer.len() > MAX_OIDC_ISSUER_BYTES
        || subject.is_empty()
        || subject.len() > MAX_OIDC_SUBJECT_BYTES
    {
        return Err(AccountError::OidcTokenValidation(anyhow::anyhow!(
            "identity claim exceeded its storage boundary"
        )));
    }
    Ok(OidcIdentityClaims {
        issuer: issuer.to_owned(),
        subject: subject.to_owned(),
        provider_email: provider_email.to_owned(),
    })
}

fn verify_access_token_hash(
    id_token: &openidconnect::core::CoreIdToken,
    claims: &openidconnect::core::CoreIdTokenClaims,
    token_response: &openidconnect::core::CoreTokenResponse,
    verifier: &openidconnect::core::CoreIdTokenVerifier<'_>,
) -> Result<(), AccountError> {
    let Some(expected_hash) = claims.access_token_hash() else {
        return Ok(());
    };
    let signing_algorithm = id_token
        .signing_alg()
        .map_err(|error| AccountError::OidcTokenValidation(anyhow::Error::new(error)))?;
    let signing_key = id_token
        .signing_key(verifier)
        .map_err(|error| AccountError::OidcTokenValidation(anyhow::Error::new(error)))?;
    let actual_hash = AccessTokenHash::from_token(
        token_response.access_token(),
        signing_algorithm,
        signing_key,
    )
    .map_err(|error| AccountError::OidcTokenValidation(anyhow::Error::new(error)))?;
    if actual_hash == *expected_hash {
        Ok(())
    } else {
        Err(AccountError::OidcTokenValidation(anyhow::anyhow!(
            "access-token hash mismatch"
        )))
    }
}
