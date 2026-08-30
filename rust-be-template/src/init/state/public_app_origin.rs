//! Canonical browser origin shared by callbacks and emailed account links.

use std::{net::IpAddr, sync::Arc};

use super::DeploymentEnvironment;

const PUBLIC_APP_ORIGIN_ENV: &str = "PUBLIC_APP_ORIGIN";

#[derive(Clone)]
pub struct PublicAppOrigin(Arc<str>);

impl PublicAppOrigin {
    pub fn from_environment(deployment: DeploymentEnvironment) -> anyhow::Result<Self> {
        let configured = match std::env::var(PUBLIC_APP_ORIGIN_ENV) {
            Ok(value) => Some(value),
            Err(std::env::VarError::NotPresent) => None,
            Err(std::env::VarError::NotUnicode(_)) => {
                return Err(anyhow::anyhow!("PUBLIC_APP_ORIGIN must contain UTF-8"));
            }
        };
        Self::parse(configured.as_deref(), deployment)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }

    pub fn as_arc(&self) -> Arc<str> {
        Arc::clone(&self.0)
    }

    pub(crate) fn parse(
        configured: Option<&str>,
        deployment: DeploymentEnvironment,
    ) -> anyhow::Result<Self> {
        let origin = match (configured.map(str::trim), deployment) {
            (Some(origin), _) if !origin.is_empty() => origin,
            (Some(_), _) => {
                return Err(anyhow::anyhow!("PUBLIC_APP_ORIGIN must not be empty"));
            }
            (None, DeploymentEnvironment::Local) => "https://localhost:30737",
            (None, DeploymentEnvironment::Prod) => "https://cyhdev.com",
            (None, DeploymentEnvironment::Dev | DeploymentEnvironment::Staging) => {
                return Err(anyhow::anyhow!(
                    "PUBLIC_APP_ORIGIN is required for development and staging"
                ));
            }
        };
        let url = reqwest::Url::parse(origin)
            .map_err(|error| anyhow::anyhow!("PUBLIC_APP_ORIGIN is invalid: {error}"))?;
        if !url.username().is_empty()
            || url.password().is_some()
            || url.host_str().is_none()
            || url.path() != "/"
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err(anyhow::anyhow!(
                "PUBLIC_APP_ORIGIN must be an exact origin without user information, path, query, or fragment"
            ));
        }
        match url.scheme() {
            "https" => {}
            "http" if matches!(deployment, DeploymentEnvironment::Local) && is_loopback(&url) => {}
            _ => {
                return Err(anyhow::anyhow!(
                    "PUBLIC_APP_ORIGIN must use HTTPS; local mode also permits loopback HTTP"
                ));
            }
        }
        Ok(Self(Arc::from(url.origin().ascii_serialization())))
    }
}

fn is_loopback(url: &reqwest::Url) -> bool {
    let host = match url.host_str() {
        Some(host) => host.trim_start_matches('[').trim_end_matches(']'),
        None => return false,
    };
    host.eq_ignore_ascii_case("localhost")
        || host
            .parse::<IpAddr>()
            .is_ok_and(|address| address.is_loopback())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_and_production_have_stable_defaults() -> anyhow::Result<()> {
        assert_eq!(
            PublicAppOrigin::parse(None, DeploymentEnvironment::Local)?.as_str(),
            "https://localhost:30737"
        );
        assert_eq!(
            PublicAppOrigin::parse(None, DeploymentEnvironment::Prod)?.as_str(),
            "https://cyhdev.com"
        );
        Ok(())
    }

    #[test]
    fn nonterminal_environments_require_an_explicit_origin() {
        assert!(PublicAppOrigin::parse(None, DeploymentEnvironment::Dev).is_err());
        assert!(PublicAppOrigin::parse(None, DeploymentEnvironment::Staging).is_err());
    }

    #[test]
    fn only_local_loopback_may_use_plain_http() {
        assert!(
            PublicAppOrigin::parse(Some("http://localhost:3000"), DeploymentEnvironment::Local)
                .is_ok()
        );
        assert!(
            PublicAppOrigin::parse(Some("http://app.example"), DeploymentEnvironment::Local)
                .is_err()
        );
        assert!(
            PublicAppOrigin::parse(Some("http://localhost:3000"), DeploymentEnvironment::Prod)
                .is_err()
        );
    }
}
