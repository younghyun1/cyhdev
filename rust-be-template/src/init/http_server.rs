//! Protocol time bounds and connection admission shared by the HTTPS and redirect listeners.
//!
//! Hyper enforces its HTTP/1 header-read timeout and HTTP/2 keep-alive pings only when a
//! timer is installed; axum-server's default builder has none. The header-read timer
//! also closes idle HTTP/1 keep-alive connections, because hyper arms it whenever it
//! waits for the next request head. HTTP/2 has no idle timer in hyper, and a client
//! that completes TLS but never sends a byte is not covered by the header timer, so
//! those connections are bounded by the per-client and global connection caps.

use std::time::Duration;

use hyper_util::{
    rt::{TokioExecutor, TokioTimer},
    server::conn::auto::Builder,
};

/// Default cap on open connections across both listeners.
pub const DEFAULT_MAX_CONNECTIONS: usize = 4_096;
/// Default cap per client IPv4 address or IPv6 /64.
pub const DEFAULT_MAX_CONNECTIONS_PER_CLIENT: usize = 64;
const MAX_CONFIGURED_CONNECTIONS: usize = 65_536;

/// Hyper time bounds; production values are [`ProtocolTimeouts::PRODUCTION`].
#[derive(Clone, Copy, Debug)]
pub struct ProtocolTimeouts {
    /// Deadline for a complete HTTP/1 request head, also the idle keep-alive limit.
    pub header_read: Duration,
    /// Interval between HTTP/2 PING frames on a quiet connection.
    pub http2_keep_alive_interval: Duration,
    /// Time allowed for a PING acknowledgement before the connection is closed.
    pub http2_keep_alive_timeout: Duration,
}

impl ProtocolTimeouts {
    pub const PRODUCTION: Self = Self {
        header_read: Duration::from_secs(20),
        http2_keep_alive_interval: Duration::from_secs(30),
        http2_keep_alive_timeout: Duration::from_secs(20),
    };
}

/// Streams per HTTP/2 connection; browsers rarely exceed 100 concurrent requests.
const HTTP2_MAX_CONCURRENT_STREAMS: u32 = 128;
/// Hyper's default; restated so a reset flood stays bounded if defaults change.
const HTTP2_MAX_PENDING_ACCEPT_RESET_STREAMS: usize = 20;
/// Request header bytes per HTTP/2 request, matching the HTTP/1 buffer order of magnitude.
const HTTP2_MAX_HEADER_LIST_SIZE: u32 = 64 * 1024;

/// Installs a Tokio timer and time bounds on both protocol builders.
pub fn configure_protocols(builder: &mut Builder<TokioExecutor>, timeouts: ProtocolTimeouts) {
    builder
        .http1()
        .timer(TokioTimer::new())
        .header_read_timeout(timeouts.header_read)
        .keep_alive(true);
    builder
        .http2()
        .timer(TokioTimer::new())
        .keep_alive_interval(timeouts.http2_keep_alive_interval)
        .keep_alive_timeout(timeouts.http2_keep_alive_timeout)
        .max_concurrent_streams(HTTP2_MAX_CONCURRENT_STREAMS)
        .max_pending_accept_reset_streams(HTTP2_MAX_PENDING_ACCEPT_RESET_STREAMS)
        .max_header_list_size(HTTP2_MAX_HEADER_LIST_SIZE);
}

/// Connection admission limits read once at startup.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HttpConnectionLimits {
    pub max_total: usize,
    pub max_per_client: usize,
}

impl HttpConnectionLimits {
    /// Reads `HTTP_MAX_CONNECTIONS` and `HTTP_MAX_CONNECTIONS_PER_IP`; invalid values
    /// abort startup rather than silently removing a bound.
    pub fn from_env() -> anyhow::Result<Self> {
        let total = optional_env("HTTP_MAX_CONNECTIONS")?;
        let per_client = optional_env("HTTP_MAX_CONNECTIONS_PER_IP")?;
        Self::parse(total.as_deref(), per_client.as_deref())
    }

    fn parse(total: Option<&str>, per_client: Option<&str>) -> anyhow::Result<Self> {
        let max_total = parse_limit("HTTP_MAX_CONNECTIONS", total, DEFAULT_MAX_CONNECTIONS)?;
        let max_per_client = parse_limit(
            "HTTP_MAX_CONNECTIONS_PER_IP",
            per_client,
            DEFAULT_MAX_CONNECTIONS_PER_CLIENT,
        )?;
        if max_per_client > max_total {
            return Err(anyhow::anyhow!(
                "HTTP_MAX_CONNECTIONS_PER_IP must not exceed HTTP_MAX_CONNECTIONS"
            ));
        }
        Ok(Self {
            max_total,
            max_per_client,
        })
    }
}

fn optional_env(name: &str) -> anyhow::Result<Option<String>> {
    match std::env::var(name) {
        Ok(value) => Ok(Some(value)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => Err(anyhow::anyhow!("{name} must be UTF-8")),
    }
}

fn parse_limit(name: &str, configured: Option<&str>, default: usize) -> anyhow::Result<usize> {
    let value = match configured.map(str::trim) {
        None | Some("") => default,
        Some(value) => value
            .parse::<usize>()
            .map_err(|error| anyhow::anyhow!("{name} must be an integer: {error}"))?,
    };
    if !(1..=MAX_CONFIGURED_CONNECTIONS).contains(&value) {
        return Err(anyhow::anyhow!(
            "{name} must be between 1 and {MAX_CONFIGURED_CONNECTIONS}"
        ));
    }
    Ok(value)
}

#[cfg(test)]
#[path = "http_server_tests.rs"]
mod tests;
