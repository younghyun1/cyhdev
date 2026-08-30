//! Strict parsing and normalization for plain HTTP benchmark targets.

use crate::error::{HarnessError, HarnessResult};

#[derive(Debug)]
pub(super) struct ParsedTarget {
    pub(super) host: String,
    pub(super) port: u16,
    pub(super) authority: String,
    pub(super) base_path: String,
}

pub(super) fn parse_target(raw_target: &str) -> HarnessResult<ParsedTarget> {
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

#[cfg(test)]
mod tests {
    use super::parse_target;

    #[test]
    fn parses_host_port_and_base_path() {
        let result = parse_target("http://127.0.0.1:3000/api/");
        assert!(result.is_ok(), "target should parse: {result:?}");
        let target = match result { Ok(target) => target, Err(_error) => return };
        assert_eq!(target.host, "127.0.0.1");
        assert_eq!(target.port, 3000);
        assert_eq!(target.base_path, "/api");
    }

    #[test]
    fn rejects_credentials_and_unsafe_paths() {
        assert!(parse_target("http://user:secret@127.0.0.1").is_err());
        assert!(parse_target("http://:3000").is_err());
        assert!(parse_target("http://127.0.0.1/api path").is_err());
        assert!(parse_target("http://127.0.0.1/api\tpath").is_err());
    }
}
