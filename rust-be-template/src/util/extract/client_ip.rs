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
        let ip = ip.to_canonical();
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
///
/// Every returned address is canonical: an IPv4-mapped IPv6 address such as
/// `::ffff:192.0.2.1`, which dual-stack listeners and some proxies report, becomes
/// the plain IPv4 address, so bans, rate limits, and visitor rows use one key per
/// client.
pub fn extract_client_ip(headers: &HeaderMap, fallback: SocketAddr) -> Option<IpAddr> {
    let config = TRUSTED_PROXIES.get_or_init(TrustedProxyConfig::from_environment);
    Some(resolve_client_ip(headers, fallback, config))
}

fn resolve_client_ip(
    headers: &HeaderMap,
    fallback: SocketAddr,
    config: &TrustedProxyConfig,
) -> IpAddr {
    let peer = fallback.ip().to_canonical();
    if config.hops == 0 || !config.trusts(peer) {
        return peer;
    }

    let raw = match forwarded_for_chain(headers) {
        Some(raw) => raw,
        None => return peer,
    };
    let mut right_to_left = raw.rsplit(',').map(str::trim);
    for _ in 1..config.hops {
        let trusted_hop = match right_to_left.next().and_then(parse_hop) {
            Some(ip) => ip,
            None => return peer,
        };
        if !config.trusts(trusted_hop) {
            return peer;
        }
    }
    right_to_left.next().and_then(parse_hop).unwrap_or(peer)
}

/// Joins every `X-Forwarded-For` field line in received order.
///
/// RFC 9110 permits a list header to arrive as several field lines, and a proxy
/// may append its own line instead of extending the first one. Reading only the
/// first line would treat an attacker-supplied leading line as the proxy's hop.
/// The combined value keeps the existing byte bound; an oversized or non-UTF-8
/// chain falls back to the socket peer.
fn forwarded_for_chain(headers: &HeaderMap) -> Option<String> {
    let mut chain = String::new();
    for value in headers.get_all("x-forwarded-for") {
        let line = value.to_str().ok()?;
        if !chain.is_empty() {
            chain.push(',');
        }
        chain.push_str(line);
        if chain.len() > MAX_FORWARDED_FOR_BYTES {
            return None;
        }
    }
    (!chain.is_empty()).then_some(chain)
}

fn parse_hop(value: &str) -> Option<IpAddr> {
    value.parse::<IpAddr>().ok().map(|ip| ip.to_canonical())
}

fn parse_trusted_networks(raw: &str) -> Vec<IpNet> {
    let mut networks = Vec::new();
    for value in raw
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
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
    use std::{
        error::Error,
        net::{IpAddr, SocketAddr},
    };

    type TestResult = Result<(), Box<dyn Error>>;

    fn two_hop_config() -> Result<TrustedProxyConfig, Box<dyn Error>> {
        Ok(TrustedProxyConfig {
            hops: 2,
            networks: vec!["10.0.0.0/8".parse::<IpNet>()?],
        })
    }

    #[test]
    fn forwarded_chain_requires_trusted_socket_and_intermediate_hops() -> TestResult {
        let config = two_hop_config()?;
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
        Ok(())
    }

    #[test]
    fn forwarded_chain_reads_every_field_line_in_order() -> TestResult {
        let config = two_hop_config()?;
        let mut headers = HeaderMap::new();
        // The client-controlled first line is spoofed; the proxy appended the
        // real client and its own hop as separate field lines.
        headers.append("x-forwarded-for", HeaderValue::from_static("198.51.100.66"));
        headers.append("x-forwarded-for", HeaderValue::from_static("192.0.2.9"));
        headers.append("x-forwarded-for", HeaderValue::from_static("10.1.2.3"));
        let peer = SocketAddr::from(([10, 9, 8, 7], 443));
        assert_eq!(
            resolve_client_ip(&headers, peer, &config),
            IpAddr::from([192, 0, 2, 9]),
        );
        Ok(())
    }

    #[test]
    fn oversized_or_invalid_multi_line_chain_falls_back_to_peer() -> TestResult {
        let config = two_hop_config()?;
        let peer = SocketAddr::from(([10, 9, 8, 7], 443));
        let mut oversized = HeaderMap::new();
        let long_line = HeaderValue::from_str(&"192.0.2.1,".repeat(150))?;
        oversized.append("x-forwarded-for", long_line.clone());
        oversized.append("x-forwarded-for", long_line);
        assert_eq!(resolve_client_ip(&oversized, peer, &config), peer.ip());

        let mut invalid = HeaderMap::new();
        invalid.append("x-forwarded-for", HeaderValue::from_static("192.0.2.9"));
        invalid.append("x-forwarded-for", HeaderValue::from_bytes(b"\xff10.1.2.3")?);
        assert_eq!(resolve_client_ip(&invalid, peer, &config), peer.ip());
        Ok(())
    }

    #[test]
    fn ipv4_mapped_addresses_are_canonicalized() -> TestResult {
        let config = two_hop_config()?;
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("::ffff:192.0.2.9, ::ffff:10.1.2.3"),
        );
        // The mapped peer and mapped hop still match the IPv4 trusted network.
        let mapped_peer = "[::ffff:10.9.8.7]:443".parse::<SocketAddr>()?;
        assert_eq!(
            resolve_client_ip(&headers, mapped_peer, &config),
            IpAddr::from([192, 0, 2, 9]),
        );

        let no_proxy = TrustedProxyConfig {
            hops: 0,
            networks: Vec::new(),
        };
        let mapped_untrusted = "[::ffff:203.0.113.4]:443".parse::<SocketAddr>()?;
        assert_eq!(
            resolve_client_ip(&HeaderMap::new(), mapped_untrusted, &no_proxy),
            IpAddr::from([203, 0, 113, 4]),
        );
        let native_v6 = "[2001:db8::1]:443".parse::<SocketAddr>()?;
        assert_eq!(
            resolve_client_ip(&HeaderMap::new(), native_v6, &no_proxy),
            native_v6.ip(),
        );
        Ok(())
    }
}
