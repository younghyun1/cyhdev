use std::{net::IpAddr, sync::Arc};

use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, Method, Request as HttpRequest, header},
    middleware::Next,
    response::IntoResponse,
};

use crate::{
    errors::code_error::{CodeError, HandlerResponse, code_err},
    init::state::{DeploymentEnvironment, PublicAppOrigin},
};

const TRUSTED_BROWSER_ORIGINS_ENV: &str = "TRUSTED_BROWSER_ORIGINS";

/// Exact browser origins permitted to issue state-changing requests or open WebSockets.
#[derive(Clone, Debug)]
pub struct TrustedOrigins {
    values: Arc<[HeaderValue]>,
}

impl TrustedOrigins {
    /// Builds the deployment defaults plus operator-configured split-origin frontends.
    pub fn from_environment(
        environment: DeploymentEnvironment,
        public_origin: &PublicAppOrigin,
    ) -> anyhow::Result<Self> {
        let mut origins = vec![public_origin.as_str().to_owned()];

        match std::env::var(TRUSTED_BROWSER_ORIGINS_ENV) {
            Ok(configured) => origins.extend(
                configured
                    .split(',')
                    .map(str::trim)
                    .filter(|origin| !origin.is_empty())
                    .map(str::to_owned),
            ),
            Err(std::env::VarError::NotPresent) => {}
            Err(error) => {
                return Err(anyhow::anyhow!(
                    "Could not read {TRUSTED_BROWSER_ORIGINS_ENV}: {error}"
                ));
            }
        }

        Self::from_origin_strings(origins, matches!(environment, DeploymentEnvironment::Local))
    }

    /// Returns the validated values used by both CORS and request-origin enforcement.
    pub fn header_values(&self) -> &[HeaderValue] {
        &self.values
    }

    fn from_origin_strings(
        origins: Vec<String>,
        allow_loopback_http: bool,
    ) -> anyhow::Result<Self> {
        let mut values = Vec::with_capacity(origins.len());
        for origin in origins {
            let value = validate_origin(&origin, allow_loopback_http).map_err(|error| {
                anyhow::anyhow!("Invalid trusted browser origin {origin:?}: {error}")
            })?;
            if !values.contains(&value) {
                values.push(value);
            }
        }

        Ok(Self {
            values: Arc::from(values),
        })
    }

    fn authorizes<B>(&self, request: &HttpRequest<B>) -> bool {
        if !requires_trusted_origin(request.method(), request.headers()) {
            return true;
        }

        let mut submitted = request.headers().get_all(header::ORIGIN).iter();
        let origin = match submitted.next() {
            Some(origin) => origin,
            None => return false,
        };
        if submitted.next().is_some() {
            return false;
        }

        self.values.contains(origin)
    }
}

/// Rejects cross-origin side effects and cross-origin WebSocket handshakes.
pub async fn require_trusted_origin(
    State(origins): State<Arc<TrustedOrigins>>,
    request: Request<Body>,
    next: Next,
) -> HandlerResponse<impl IntoResponse> {
    if !origins.authorizes(&request) {
        return Err(code_err(
            CodeError::UNTRUSTED_ORIGIN,
            "Origin header is missing, duplicated, or not trusted",
        ));
    }

    Ok(next.run(request).await)
}

fn requires_trusted_origin(method: &Method, headers: &HeaderMap) -> bool {
    let is_safe_method =
        method == Method::GET || method == Method::HEAD || method == Method::OPTIONS;
    !is_safe_method || is_websocket_upgrade(headers)
}

fn is_websocket_upgrade(headers: &HeaderMap) -> bool {
    headers
        .get_all(header::UPGRADE)
        .iter()
        .flat_map(|value| value.as_bytes().split(|byte| *byte == b','))
        .any(|protocol| protocol.trim_ascii().eq_ignore_ascii_case(b"websocket"))
}

fn validate_origin(origin: &str, allow_loopback_http: bool) -> Result<HeaderValue, &'static str> {
    let url = reqwest::Url::parse(origin).map_err(|_| "origin is not an absolute HTTP URL")?;
    let scheme = url.scheme();
    if !matches!(scheme, "http" | "https") {
        return Err("origin scheme must be HTTP or HTTPS");
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("origin must not include user information");
    }
    if url.path() != "/" || url.query().is_some() || url.fragment().is_some() {
        return Err("origin must not include a path, query, or fragment");
    }
    if url.host_str().is_none() {
        return Err("origin does not include a host");
    }
    if scheme == "http" && (!allow_loopback_http || !has_loopback_host(&url)) {
        return Err("plain HTTP is allowed only for local loopback origins");
    }

    let canonical = url.origin().ascii_serialization();
    HeaderValue::from_str(&canonical).map_err(|_| "origin is not a valid HTTP header value")
}

fn has_loopback_host(url: &reqwest::Url) -> bool {
    let host = match url.host_str() {
        Some(host) => host.trim_start_matches('[').trim_end_matches(']'),
        None => return false,
    };
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }
    match host.parse::<IpAddr>() {
        Ok(address) => address.is_loopback(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::Request};

    use super::*;

    fn trusted_origins() -> anyhow::Result<TrustedOrigins> {
        TrustedOrigins::from_origin_strings(vec!["https://cyhdev.com".to_string()], true)
    }

    #[test]
    fn unsafe_requests_require_one_exact_origin() -> anyhow::Result<()> {
        let origins = trusted_origins()?;
        let trusted = Request::builder()
            .method(Method::POST)
            .header(header::ORIGIN, "https://cyhdev.com")
            .body(Body::empty())?;
        let untrusted = Request::builder()
            .method(Method::POST)
            .header(header::ORIGIN, "https://attacker.example")
            .body(Body::empty())?;
        let missing = Request::builder()
            .method(Method::POST)
            .body(Body::empty())?;
        let duplicated = Request::builder()
            .method(Method::POST)
            .header(header::ORIGIN, "https://cyhdev.com")
            .header(header::ORIGIN, "https://cyhdev.com")
            .body(Body::empty())?;

        assert!(origins.authorizes(&trusted));
        assert!(!origins.authorizes(&untrusted));
        assert!(!origins.authorizes(&missing));
        assert!(!origins.authorizes(&duplicated));
        Ok(())
    }

    #[test]
    fn websocket_get_requires_a_trusted_origin() -> anyhow::Result<()> {
        let origins = trusted_origins()?;
        let websocket = Request::builder()
            .method(Method::GET)
            .header(header::UPGRADE, "websocket")
            .header(header::ORIGIN, "https://attacker.example")
            .body(Body::empty())?;
        let ordinary_get = Request::builder().method(Method::GET).body(Body::empty())?;

        assert!(!origins.authorizes(&websocket));
        assert!(origins.authorizes(&ordinary_get));
        Ok(())
    }

    #[test]
    fn production_configuration_rejects_plain_http() {
        assert!(validate_origin("http://cyhdev.com", false).is_err());
        assert!(validate_origin("https://cyhdev.com/path", false).is_err());
        assert!(validate_origin("https://user@cyhdev.com", false).is_err());
        assert!(validate_origin("https://cyhdev.com?query", false).is_err());
        assert!(validate_origin("https://cyhdev.com#fragment", false).is_err());
        assert!(validate_origin("*", false).is_err());
        assert!(validate_origin("null", false).is_err());
        assert!(validate_origin("https://cyhdev.com", false).is_ok());
    }

    #[test]
    fn canonicalizes_host_case_and_default_ports() {
        assert_eq!(
            validate_origin("https://CYHDEV.com:443", false).ok(),
            Some(HeaderValue::from_static("https://cyhdev.com"))
        );
    }

    #[test]
    fn local_plain_http_is_limited_to_loopback() {
        assert!(validate_origin("http://localhost:3000", true).is_ok());
        assert!(validate_origin("http://127.0.0.2:3000", true).is_ok());
        assert!(validate_origin("http://[::1]:3000", true).is_ok());
        assert!(validate_origin("http://dev.example:3000", true).is_err());
    }

    #[test]
    fn any_websocket_upgrade_value_requires_an_origin() -> anyhow::Result<()> {
        let origins = trusted_origins()?;
        let websocket = Request::builder()
            .method(Method::GET)
            .header(header::UPGRADE, "h2c")
            .header(header::UPGRADE, "websocket")
            .body(Body::empty())?;

        assert!(!origins.authorizes(&websocket));
        Ok(())
    }
}
