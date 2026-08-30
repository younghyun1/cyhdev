//! Plain-HTTP executor for an explicitly supplied local or staged target.

use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream, ToSocketAddrs},
    time::Duration,
};

use crate::{
    config::RequestSpec,
    error::{HarnessError, HarnessResult},
    executor::types::{ExecutionOutcome, RequestFailure},
};

#[derive(Clone, Debug)]
pub struct HttpExecutor {
    address: SocketAddr,
    authority: String,
    base_path: String,
    timeout: Duration,
    max_response_bytes: usize,
}

impl HttpExecutor {
    pub fn new(raw_target: &str, timeout: Duration, max_response_bytes: usize) -> HarnessResult<Self> {
        let target = parse_target(raw_target)?;
        let mut addresses = (target.host.as_str(), target.port)
            .to_socket_addrs()
            .map_err(|source| HarnessError::Resolve {
                target: raw_target.to_owned(),
                source,
            })?;
        let address = match addresses.next() {
            Some(address) => address,
            None => {
                return Err(HarnessError::Arguments(format!(
                    "HTTP target `{raw_target}` resolved to no addresses"
                )));
            }
        };
        Ok(Self {
            address,
            authority: target.authority,
            base_path: target.base_path,
            timeout,
            max_response_bytes,
        })
    }

    pub fn execute(&self, request: &RequestSpec) -> Result<ExecutionOutcome, RequestFailure> {
        let mut stream = TcpStream::connect_timeout(&self.address, self.timeout)
            .map_err(|_source| RequestFailure::Connect)?;
        stream
            .set_read_timeout(Some(self.timeout))
            .and_then(|()| stream.set_write_timeout(Some(self.timeout)))
            .map_err(|_source| RequestFailure::ConfigureSocket)?;

        let path = self.request_path(&request.path);
        let wire_request = format!(
            "{} {} HTTP/1.1\r\nHost: {}\r\nAccept: application/json\r\nConnection: close\r\nUser-Agent: cyhdev-throughput/1\r\n\r\n",
            request.method, path, self.authority
        );
        stream
            .write_all(wire_request.as_bytes())
            .map_err(|_source| RequestFailure::Write)?;

        let response = self.read_response(&mut stream)?;
        let status = parse_status(&response)?;
        Ok(ExecutionOutcome {
            status,
            response_bytes: response.len(),
            checksum: hash_bytes(&response),
        })
    }

    pub fn label(&self) -> String {
        format!("http://{}{}", self.authority, self.base_path)
    }

    pub fn resolved_address(&self) -> String {
        self.address.to_string()
    }

    fn request_path(&self, request_path: &str) -> String {
        if self.base_path == "/" {
            request_path.to_owned()
        } else {
            format!("{}{}", self.base_path.trim_end_matches('/'), request_path)
        }
    }

    fn read_response(&self, stream: &mut TcpStream) -> Result<Vec<u8>, RequestFailure> {
        let initial_capacity = self.max_response_bytes.min(8 * 1024);
        let mut response = Vec::with_capacity(initial_capacity);
        let mut chunk = [0_u8; 8 * 1024];
        loop {
            let count = stream
                .read(&mut chunk)
                .map_err(|_source| RequestFailure::Read)?;
            if count == 0 {
                break;
            }
            let new_length = match response.len().checked_add(count) {
                Some(length) => length,
                None => return Err(RequestFailure::ResponseTooLarge),
            };
            if new_length > self.max_response_bytes {
                return Err(RequestFailure::ResponseTooLarge);
            }
            response.extend_from_slice(&chunk[..count]);
        }
        Ok(response)
    }
}

#[derive(Debug)]
struct ParsedTarget {
    host: String,
    port: u16,
    authority: String,
    base_path: String,
}

fn parse_target(raw_target: &str) -> HarnessResult<ParsedTarget> {
    let without_scheme = match raw_target.strip_prefix("http://") {
        Some(value) => value,
        None => {
            return Err(HarnessError::Arguments(
                "--target must use plain `http://`; terminate TLS before this local harness"
                    .to_owned(),
            ));
        }
    };
    if without_scheme.contains('@') || without_scheme.contains('?') || without_scheme.contains('#') {
        return Err(HarnessError::Arguments(
            "--target must not contain credentials, a query, a fragment, or control characters"
                .to_owned(),
        ));
    }
    let (authority, path) = match without_scheme.split_once('/') {
        Some((authority, path)) => (authority, format!("/{path}")),
        None => (without_scheme, "/".to_owned()),
    };
    if authority.is_empty()
        || !authority.is_ascii()
        || authority
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte.is_ascii_whitespace())
    {
        return Err(HarnessError::Arguments(
            "--target must contain a valid host".to_owned(),
        ));
    }
    if !path.is_ascii()
        || path
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte.is_ascii_whitespace())
    {
        return Err(HarnessError::Arguments(
            "--target base path must be ASCII and contain no whitespace or control characters"
                .to_owned(),
        ));
    }
    let (host, port) = parse_authority(authority)?;
    if host.is_empty() {
        return Err(HarnessError::Arguments(
            "--target must contain a nonempty host".to_owned(),
        ));
    }
    let trimmed_path = path.trim_end_matches('/');
    let base_path = if trimmed_path.is_empty() {
        "/".to_owned()
    } else {
        trimmed_path.to_owned()
    };
    Ok(ParsedTarget {
        host,
        port,
        authority: authority.to_owned(),
        base_path,
    })
}

fn parse_authority(authority: &str) -> HarnessResult<(String, u16)> {
    if let Some(bracketed) = authority.strip_prefix('[') {
        let (host, suffix) = match bracketed.split_once(']') {
            Some(parts) => parts,
            None => {
                return Err(HarnessError::Arguments(
                    "IPv6 targets must close the host with `]`".to_owned(),
                ));
            }
        };
        let port = parse_optional_port(suffix)?;
        return Ok((host.to_owned(), port));
    }
    match authority.rsplit_once(':') {
        Some((host, port)) if !host.contains(':') => Ok((host.to_owned(), parse_port(port)?)),
        Some((_host, _port)) => Err(HarnessError::Arguments(
            "IPv6 targets must enclose the host in brackets".to_owned(),
        )),
        None => Ok((authority.to_owned(), 80)),
    }
}

fn parse_optional_port(suffix: &str) -> HarnessResult<u16> {
    if suffix.is_empty() {
        Ok(80)
    } else {
        match suffix.strip_prefix(':') {
            Some(port) => parse_port(port),
            None => Err(HarnessError::Arguments(
                "unexpected characters after the IPv6 host".to_owned(),
            )),
        }
    }
}

fn parse_port(port: &str) -> HarnessResult<u16> {
    match port.parse::<u16>() {
        Ok(0) | Err(_) => Err(HarnessError::Arguments(
            "--target port must be in 1..=65535".to_owned(),
        )),
        Ok(port) => Ok(port),
    }
}

fn parse_status(response: &[u8]) -> Result<u16, RequestFailure> {
    let header_end = response
        .windows(2)
        .position(|window| window == b"\r\n")
        .ok_or(RequestFailure::InvalidResponse)?;
    let status_line = std::str::from_utf8(&response[..header_end])
        .map_err(|_source| RequestFailure::InvalidResponse)?;
    let mut fields = status_line.split_ascii_whitespace();
    let protocol = fields.next().ok_or(RequestFailure::InvalidResponse)?;
    let status = fields.next().ok_or(RequestFailure::InvalidResponse)?;
    if !protocol.starts_with("HTTP/1.") {
        return Err(RequestFailure::InvalidResponse);
    }
    status
        .parse::<u16>()
        .map_err(|_source| RequestFailure::InvalidResponse)
}

fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::parse_target;

    #[test]
    fn parses_host_port_and_base_path() {
        let result = parse_target("http://127.0.0.1:3000/api/");
        assert!(result.is_ok(), "target should parse: {result:?}");
        let target = match result {
            Ok(target) => target,
            Err(_error) => return,
        };
        assert_eq!(target.host, "127.0.0.1");
        assert_eq!(target.port, 3000);
        assert_eq!(target.base_path, "/api");
    }

    #[test]
    fn rejects_credentials() {
        assert!(parse_target("http://user:secret@127.0.0.1").is_err());
    }

    #[test]
    fn rejects_empty_hosts_and_unsafe_base_paths() {
        assert!(parse_target("http://:3000").is_err());
        assert!(parse_target("http://127.0.0.1/api path").is_err());
        assert!(parse_target("http://127.0.0.1/api\tpath").is_err());
    }
}
