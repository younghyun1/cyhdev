use std::{net::IpAddr, net::SocketAddr, sync::OnceLock};

use axum::http::HeaderMap;
use ipnet::IpNet;

const MAX_TRUSTED_PROXY_HOPS: usize = 16;
const MAX_FORWARDED_FOR_BYTES: usize = 2_048;
static TRUSTED_PROXIES: OnceLock<TrustedProxyConfig> = OnceLock::new();

struct TrustedProxyConfig {
    hops: usize,
    networks: Vec<IpNet>,
}

impl TrustedProxyConfig {
    fn from_environment() -> Self {
        let hops = std::env::var("TRUSTED_PROXY_HOPS")
            .ok()
            .and_then(|value| value.trim().parse::<usize>().ok())
            .filter(|hops| *hops <= MAX_TRUSTED_PROXY_HOPS)
            .unwrap_or(0);
        let networks = match std::env::var("TRUSTED_PROXY_CIDRS") {
            Ok(value) => parse_trusted_networks(&value),
            Err(_) => Vec::new(),
        };
        if hops > 0 && networks.is_empty() {
            tracing::warn!(
                "Forwarded client IPs disabled because no valid trusted proxy CIDR is configured"
            );
        }
        Self { hops, networks }
    }

    fn trusts(&self, ip: IpAddr) -> bool {
        self.networks.iter().any(|network| network.contains(&ip))
    }
}

/// Resolve the real client IP, establishing a trusted-proxy boundary instead of
/// trusting the leftmost (attacker-controlled) `X-Forwarded-For` hop.
///
/// We select the rightmost UNTRUSTED address from the hop chain, gated on the
/// configured trusted-hop count. When no trusted proxy boundary is configured we
/// ignore client-supplied headers entirely and use the socket peer; this is the
/// fail-safe default for ban enforcement and visitor logging.
pub fn extract_client_ip(headers: &HeaderMap, fallback: SocketAddr) -> Option<IpAddr> {
    let config = TRUSTED_PROXIES.get_or_init(TrustedProxyConfig::from_environment);
    Some(resolve_client_ip(headers, fallback, config))
}

fn resolve_client_ip(
    headers: &HeaderMap,
    fallback: SocketAddr,
    config: &TrustedProxyConfig,
) -> IpAddr {
    if config.hops == 0 || !config.trusts(fallback.ip()) {
        return fallback.ip();
    }

    let raw = match headers.get("x-forwarded-for") {
        Some(value) if value.as_bytes().len() <= MAX_FORWARDED_FOR_BYTES => match value.to_str() {
            Ok(raw) => raw,
            Err(_) => return fallback.ip(),
        },
        Some(_) | None => return fallback.ip(),
    };
    let mut right_to_left = raw.rsplit(',').map(str::trim);
    for _ in 1..config.hops {
        let trusted_hop = match right_to_left.next().and_then(|value| value.parse().ok()) {
            Some(ip) => ip,
            None => return fallback.ip(),
        };
        if !config.trusts(trusted_hop) {
            return fallback.ip();
        }
    }
    match right_to_left.next().and_then(|value| value.parse::<IpAddr>().ok()) {
        Some(client_ip) => client_ip,
        None => fallback.ip(),
    }
}

fn parse_trusted_networks(raw: &str) -> Vec<IpNet> {
    let mut networks = Vec::new();
    for value in raw.split(',').map(str::trim).filter(|value| !value.is_empty()) {
        match value.parse::<IpNet>() {
            Ok(network) => networks.push(network),
            Err(error) => {
                tracing::warn!(error = %error, "Forwarded client IPs disabled by invalid proxy CIDR");
                return Vec::new();
            }
        }
    }
    networks
}

#[cfg(test)]
mod tests {
    use super::{TrustedProxyConfig, resolve_client_ip};
    use axum::http::{HeaderMap, HeaderValue};
    use ipnet::IpNet;
    use std::net::{IpAddr, SocketAddr};

    #[test]
    fn forwarded_chain_requires_trusted_socket_and_intermediate_hops() {
        let network = match "10.0.0.0/8".parse::<IpNet>() {
            Ok(network) => network,
            Err(error) => panic!("static trusted network is invalid: {error}"),
        };
        let config = TrustedProxyConfig {
            hops: 2,
            networks: vec![network],
        };
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("192.0.2.9, 10.1.2.3"),
        );
        let peer = SocketAddr::from(([10, 9, 8, 7], 443));
        assert_eq!(
            resolve_client_ip(&headers, peer, &config),
            IpAddr::from([192, 0, 2, 9]),
        );

        let untrusted_peer = SocketAddr::from(([203, 0, 113, 4], 443));
        assert_eq!(
            resolve_client_ip(&headers, untrusted_peer, &config),
            untrusted_peer.ip(),
        );
    }
}
