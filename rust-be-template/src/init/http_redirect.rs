//! Plain-HTTP listener that only redirects to the HTTPS origin.
//!
//! It shares the HTTPS listener's connection and request budgets and hyper time
//! bounds, so port 80 cannot provide a second budget for abusive clients.

use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use axum::{
    Router,
    http::{StatusCode, Uri, uri::Authority},
    response::Redirect,
};
use axum_server::{Handle, accept::DefaultAcceptor};

use crate::{
    init::{
        connection_acceptor::LimitedAcceptor,
        http_server::{ProtocolTimeouts, configure_protocols},
    },
    routers::middleware::request_rate_limit,
    util::{
        connection_limit::ConnectionLimiter, extract::Host, request_rate_limit::RequestRateLimiter,
    },
};

#[derive(Clone, Copy)]
pub struct Ports {
    pub http: u16,
    pub https: u16,
}

/// Allow disposable runtimes to use an unprivileged redirect listener.
pub(super) fn port_from_environment() -> anyhow::Result<u16> {
    match std::env::var("HTTP_REDIRECT_PORT") {
        Ok(value) => parse_port(&value),
        Err(std::env::VarError::NotPresent) => Ok(80),
        Err(error) => Err(anyhow::anyhow!("HTTP_REDIRECT_PORT is invalid: {error}")),
    }
}

fn parse_port(value: &str) -> anyhow::Result<u16> {
    match value.parse::<u16>() {
        Ok(port) if port != 0 => Ok(port),
        _ => Err(anyhow::anyhow!(
            "HTTP_REDIRECT_PORT must be an integer between 1 and 65535"
        )),
    }
}

/// Serves redirects until `handle` requests shutdown.
pub async fn redirect_http_to_https(
    host_ip: IpAddr,
    ports: Ports,
    handle: Handle<SocketAddr>,
    limiter: ConnectionLimiter,
    request_limiter: Arc<RequestRateLimiter>,
) -> anyhow::Result<()> {
    let redirect = move |Host(host): Host, uri: Uri| async move {
        match make_https(&host, uri, ports.https) {
            Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
            Err(error) => {
                tracing::warn!(error = %error, "Failed to convert URI to HTTPS");
                Err(StatusCode::BAD_REQUEST)
            }
        }
    };

    let router = request_rate_limit::apply(Router::new().fallback(redirect), request_limiter);
    let addr = SocketAddr::new(host_ip, ports.http);
    let mut server = axum_server::bind(addr)
        .acceptor(LimitedAcceptor::new(DefaultAcceptor::new(), limiter))
        .handle(handle);
    configure_protocols(server.http_builder(), ProtocolTimeouts::PRODUCTION);
    tracing::debug!(local_addr = %addr, "Listening for HTTP redirect traffic");
    server
        .serve(router.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to serve redirection: {}", e))
}

fn make_https(host: &str, uri: Uri, https_port: u16) -> anyhow::Result<Uri> {
    let mut parts = uri.into_parts();

    parts.scheme = Some(axum::http::uri::Scheme::HTTPS);

    if parts.path_and_query.is_none() {
        parts.path_and_query = Some(
            "/".parse()
                .map_err(|e| anyhow::anyhow!("Failed to parse '/' as path: {}", e))?,
        );
    }

    let authority: Authority = host
        .parse()
        .map_err(|e| anyhow::anyhow!("Failed to parse host into Authority: {}", e))?;
    let bare_host = match authority.port() {
        Some(port_struct) => authority
            .as_str()
            .strip_suffix(port_struct.as_str())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Failed to remove port ({}) from authority string",
                    port_struct
                )
            })?
            .strip_suffix(':')
            .ok_or_else(|| anyhow::anyhow!("Failed to remove colon from authority string"))?,
        None => authority.as_str(),
    };

    parts.authority = Some(
        format!("{bare_host}:{https_port}")
            .parse()
            .map_err(|e| anyhow::anyhow!("Failed to parse new authority: {}", e))?,
    );

    Uri::from_parts(parts).map_err(|e| anyhow::anyhow!("Failed to construct HTTPS URI: {}", e))
}

#[cfg(test)]
mod port_tests {
    #[test]
    fn accepts_default_and_disposable_listener_ports() -> anyhow::Result<()> {
        assert_eq!(super::parse_port("80")?, 80);
        assert_eq!(super::parse_port("18444")?, 18444);
        assert_eq!(super::parse_port("65535")?, 65535);
        Ok(())
    }

    #[test]
    fn rejects_zero_overflow_and_malformed_ports() {
        for value in ["0", "65536", "-1", "", "not-a-port"] {
            assert!(super::parse_port(value).is_err());
        }
    }
}
