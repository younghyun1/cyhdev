//! Plain-HTTP listener that only redirects to the HTTPS origin.
//!
//! It shares the HTTPS listener's connection limiter and hyper time bounds, so port 80
//! cannot be used to hold connections the TLS listener would refuse.

use std::net::{IpAddr, SocketAddr};

use axum::{
    handler::HandlerWithoutStateExt,
    http::{StatusCode, Uri, uri::Authority},
    response::Redirect,
};
use axum_server::{Handle, accept::DefaultAcceptor};

use crate::{
    init::{
        connection_acceptor::LimitedAcceptor,
        http_server::{ProtocolTimeouts, configure_protocols},
    },
    util::{connection_limit::ConnectionLimiter, extract::Host},
};

#[derive(Clone, Copy)]
pub struct Ports {
    pub http: u16,
    pub https: u16,
}

/// Serves redirects until `handle` requests shutdown.
pub async fn redirect_http_to_https(
    host_ip: IpAddr,
    ports: Ports,
    handle: Handle<SocketAddr>,
    limiter: ConnectionLimiter,
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

    let addr = SocketAddr::new(host_ip, ports.http);
    let mut server = axum_server::bind(addr)
        .acceptor(LimitedAcceptor::new(DefaultAcceptor::new(), limiter))
        .handle(handle);
    configure_protocols(server.http_builder(), ProtocolTimeouts::PRODUCTION);
    tracing::debug!(local_addr = %addr, "Listening for HTTP redirect traffic");
    server
        .serve(redirect.into_make_service())
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
