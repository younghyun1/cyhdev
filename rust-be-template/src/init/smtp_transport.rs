//! Explicit local SMTP TLS roots and ports for isolated development providers.

use lettre::{
    AsyncSmtpTransport, Tokio1Executor,
    transport::smtp::client::{Certificate, Tls, TlsParameters},
};

use super::{config::EmailConfig, state::DeploymentEnvironment};

pub(super) fn build(config: &EmailConfig) -> anyhow::Result<AsyncSmtpTransport<Tokio1Executor>> {
    let mut builder = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.get_url())?
        .credentials(config.to_creds());
    let port = std::env::var("LOCAL_SMTP_PORT").ok();
    let root = std::env::var_os("LOCAL_SMTP_CA_PEM");
    if port.is_some() || root.is_some() {
        validate_local(DeploymentEnvironment::from_env()?, &config.get_url())?;
        if let Some(port) = port {
            let port: u16 = port.parse()?;
            anyhow::ensure!(port >= 1024, "LOCAL_SMTP_PORT must be unprivileged");
            builder = builder.port(port);
        }
        if let Some(root) = root {
            let certificate = Certificate::from_pem(&std::fs::read(root)?)?;
            let tls = TlsParameters::builder(config.get_url())
                .add_root_certificate(certificate)
                .build()?;
            // Certificate and host verification remain enabled with the additional local root.
            builder = builder.tls(Tls::Wrapper(tls));
        }
    }
    Ok(builder.build())
}

fn validate_local(environment: DeploymentEnvironment, host: &str) -> anyhow::Result<()> {
    anyhow::ensure!(
        environment == DeploymentEnvironment::Local
            && (host == "localhost"
                || host
                    .parse::<std::net::IpAddr>()
                    .is_ok_and(|ip| ip.is_loopback())),
        "local SMTP overrides require local mode and a loopback relay"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn private_roots_are_restricted_to_local_loopback_relays() {
        assert!(validate_local(DeploymentEnvironment::Local, "127.0.0.1").is_ok());
        assert!(validate_local(DeploymentEnvironment::Prod, "127.0.0.1").is_err());
        assert!(validate_local(DeploymentEnvironment::Local, "smtp.example.test").is_err());
    }
}
