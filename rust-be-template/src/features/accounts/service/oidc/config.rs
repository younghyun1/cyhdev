//! Optional, all-or-nothing OpenID Connect deployment configuration.

use std::{env, net::IpAddr, sync::Arc};

use openidconnect::{ClientId, ClientSecret, IssuerUrl, RedirectUrl};

use crate::{
    features::accounts::domain::oidc::MAX_OIDC_ISSUER_BYTES,
    init::state::{DeploymentEnvironment, PublicAppOrigin},
};

const PROVIDER_NAME_MAX_BYTES: usize = 80;
const CLIENT_ID_MAX_BYTES: usize = 512;
const CLIENT_SECRET_MAX_BYTES: usize = 4_096;

pub(super) struct OidcConfig {
    pub(super) provider_name: Arc<str>,
    pub(super) issuer: IssuerUrl,
    pub(super) client_id: ClientId,
    pub(super) client_secret: Option<ClientSecret>,
    pub(super) redirect_url: RedirectUrl,
    pub(super) allow_loopback_http: bool,
}

impl OidcConfig {
    pub(super) fn from_environment(
        deployment: DeploymentEnvironment,
        public_origin: &PublicAppOrigin,
    ) -> anyhow::Result<Option<Self>> {
        Self::parse(OidcEnvironmentValues::read()?, deployment, public_origin)
    }

    fn parse(
        values: OidcEnvironmentValues,
        deployment: DeploymentEnvironment,
        public_origin: &PublicAppOrigin,
    ) -> anyhow::Result<Option<Self>> {
        if !values.any_present() {
            return Ok(None);
        }
        let provider_name = required(values.provider_name, "OIDC_PROVIDER_NAME")?;
        let issuer = required(values.issuer, "OIDC_ISSUER_URL")?;
        let client_id = required(values.client_id, "OIDC_CLIENT_ID")?;
        validate_text(
            "OIDC_PROVIDER_NAME",
            &provider_name,
            PROVIDER_NAME_MAX_BYTES,
        )?;
        validate_text("OIDC_CLIENT_ID", &client_id, CLIENT_ID_MAX_BYTES)?;
        if let Some(secret) = &values.client_secret {
            validate_text("OIDC_CLIENT_SECRET", secret, CLIENT_SECRET_MAX_BYTES)?;
        }
        if issuer.len() > MAX_OIDC_ISSUER_BYTES {
            return Err(anyhow::anyhow!(
                "OIDC_ISSUER_URL exceeds {MAX_OIDC_ISSUER_BYTES} bytes"
            ));
        }

        let allow_loopback_http = matches!(deployment, DeploymentEnvironment::Local);
        let issuer_url = reqwest::Url::parse(&issuer)
            .map_err(|error| anyhow::anyhow!("OIDC_ISSUER_URL is invalid: {error}"))?;
        validate_remote_url(&issuer_url, allow_loopback_http, "OIDC_ISSUER_URL")?;
        if issuer_url.query().is_some() || issuer_url.fragment().is_some() {
            return Err(anyhow::anyhow!(
                "OIDC_ISSUER_URL must not include a query or fragment"
            ));
        }
        let issuer = IssuerUrl::new(issuer_url.to_string())
            .map_err(|error| anyhow::anyhow!("OIDC_ISSUER_URL is invalid: {error}"))?;

        let redirect_url =
            RedirectUrl::new(format!("{}/api/auth/oidc/callback", public_origin.as_str()))
                .map_err(|error| anyhow::anyhow!("OIDC callback URL is invalid: {error}"))?;

        Ok(Some(Self {
            provider_name: Arc::from(provider_name),
            issuer,
            client_id: ClientId::new(client_id),
            client_secret: values.client_secret.map(ClientSecret::new),
            redirect_url,
            allow_loopback_http,
        }))
    }
}

fn validate_text(key: &str, value: &str, max_bytes: usize) -> anyhow::Result<()> {
    if value.is_empty() || value.len() > max_bytes || value.chars().any(char::is_control) {
        Err(anyhow::anyhow!(
            "{key} must contain 1-{max_bytes} non-control UTF-8 bytes"
        ))
    } else {
        Ok(())
    }
}

pub(super) fn validate_remote_url(
    url: &reqwest::Url,
    allow_loopback_http: bool,
    field: &str,
) -> anyhow::Result<()> {
    if !url.username().is_empty() || url.password().is_some() || url.host_str().is_none() {
        return Err(anyhow::anyhow!(
            "{field} must contain a host and no user information"
        ));
    }
    match url.scheme() {
        "https" => Ok(()),
        "http" if allow_loopback_http && has_loopback_host(url) => Ok(()),
        _ => Err(anyhow::anyhow!(
            "{field} must use HTTPS; local mode also permits loopback HTTP"
        )),
    }
}

fn has_loopback_host(url: &reqwest::Url) -> bool {
    let host = match url.host_str() {
        Some(host) => host.trim_start_matches('[').trim_end_matches(']'),
        None => return false,
    };
    host.eq_ignore_ascii_case("localhost")
        || host
            .parse::<IpAddr>()
            .is_ok_and(|address| address.is_loopback())
}

fn required(value: Option<String>, key: &str) -> anyhow::Result<String> {
    value.ok_or_else(|| anyhow::anyhow!("{key} is required when any OIDC setting is present"))
}

#[derive(Default)]
struct OidcEnvironmentValues {
    provider_name: Option<String>,
    issuer: Option<String>,
    client_id: Option<String>,
    client_secret: Option<String>,
}

impl OidcEnvironmentValues {
    fn read() -> anyhow::Result<Self> {
        Ok(Self {
            provider_name: read_trimmed("OIDC_PROVIDER_NAME")?,
            issuer: read_trimmed("OIDC_ISSUER_URL")?,
            client_id: read_trimmed("OIDC_CLIENT_ID")?,
            client_secret: read_exact("OIDC_CLIENT_SECRET")?,
        })
    }

    fn any_present(&self) -> bool {
        self.provider_name.is_some()
            || self.issuer.is_some()
            || self.client_id.is_some()
            || self.client_secret.is_some()
    }
}

fn read_trimmed(key: &'static str) -> anyhow::Result<Option<String>> {
    match env::var(key) {
        Ok(value) => Ok(Some(value.trim().to_owned())),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => Err(anyhow::anyhow!("{key} must contain UTF-8")),
    }
}

fn read_exact(key: &'static str) -> anyhow::Result<Option<String>> {
    match env::var(key) {
        Ok(value) => Ok(Some(value)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => Err(anyhow::anyhow!("{key} must contain UTF-8")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn configured() -> OidcEnvironmentValues {
        OidcEnvironmentValues {
            provider_name: Some("Example".to_owned()),
            issuer: Some("https://id.example.test".to_owned()),
            client_id: Some("client-id".to_owned()),
            client_secret: None,
        }
    }

    #[test]
    fn absence_disables_oidc_cleanly() -> anyhow::Result<()> {
        let origin = PublicAppOrigin::parse(None, DeploymentEnvironment::Prod)?;
        assert!(
            OidcConfig::parse(
                OidcEnvironmentValues::default(),
                DeploymentEnvironment::Prod,
                &origin
            )?
            .is_none()
        );
        Ok(())
    }

    #[test]
    fn partial_configuration_is_rejected() {
        let values = OidcEnvironmentValues {
            client_id: Some("orphan-client".to_owned()),
            ..OidcEnvironmentValues::default()
        };
        let origin = PublicAppOrigin::parse(None, DeploymentEnvironment::Prod).ok();
        assert!(origin.is_some());
        if let Some(origin) = origin {
            assert!(OidcConfig::parse(values, DeploymentEnvironment::Prod, &origin).is_err());
        }
    }

    #[test]
    fn callback_is_derived_from_one_exact_public_origin() -> anyhow::Result<()> {
        let origin = PublicAppOrigin::parse(
            Some("https://APP.example.test:443"),
            DeploymentEnvironment::Prod,
        )?;
        let config = OidcConfig::parse(configured(), DeploymentEnvironment::Prod, &origin)?
            .ok_or_else(|| anyhow::anyhow!("configuration unexpectedly disabled"))?;
        assert_eq!(
            config.redirect_url.as_str(),
            "https://app.example.test/api/auth/oidc/callback"
        );
        Ok(())
    }

    #[test]
    fn plain_http_issuer_is_limited_to_local_loopback() -> anyhow::Result<()> {
        let prod_origin = PublicAppOrigin::parse(None, DeploymentEnvironment::Prod)?;
        let mut remote_http = configured();
        remote_http.issuer = Some("http://id.example.test".to_owned());
        assert!(OidcConfig::parse(remote_http, DeploymentEnvironment::Prod, &prod_origin).is_err());
        let local_origin = PublicAppOrigin::parse(None, DeploymentEnvironment::Local)?;
        let mut loopback_http = configured();
        loopback_http.issuer = Some("http://localhost:8080".to_owned());
        assert!(
            OidcConfig::parse(loopback_http, DeploymentEnvironment::Local, &local_origin).is_ok()
        );
        Ok(())
    }

    #[test]
    fn client_secret_bytes_are_not_trimmed() -> anyhow::Result<()> {
        let origin = PublicAppOrigin::parse(None, DeploymentEnvironment::Prod)?;
        let mut values = configured();
        values.client_secret = Some(" leading-and-trailing ".to_owned());
        let config = OidcConfig::parse(values, DeploymentEnvironment::Prod, &origin)?
            .ok_or_else(|| anyhow::anyhow!("configuration unexpectedly disabled"))?;
        let secret = config
            .client_secret
            .as_ref()
            .map(|secret| secret.secret().as_str());
        assert_eq!(secret, Some(" leading-and-trailing "));
        Ok(())
    }
}
