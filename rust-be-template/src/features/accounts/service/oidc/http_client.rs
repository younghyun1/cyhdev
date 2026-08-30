//! Bounded OIDC HTTP adapter backed by the workspace's existing reqwest/rustls stack.

use std::{future::Future, io, pin::Pin, time::Duration};

use openidconnect::{AsyncHttpClient, http};

use super::config::validate_remote_url;

const MAX_OIDC_REQUEST_BYTES: usize = 64 * 1024;
const MAX_OIDC_RESPONSE_BYTES: usize = 2 * 1024 * 1024;
const OIDC_HTTP_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Clone)]
pub(super) struct OidcHttpClient {
    client: reqwest::Client,
    allow_loopback_http: bool,
}

impl OidcHttpClient {
    pub(super) fn new(allow_loopback_http: bool) -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            // OIDC endpoints come from trusted discovery metadata. Redirects are
            // rejected so a compromised endpoint cannot turn that trust into SSRF.
            .redirect(reqwest::redirect::Policy::none())
            .timeout(OIDC_HTTP_TIMEOUT)
            .user_agent("cyhdev-oidc/1")
            .build()?;
        Ok(Self {
            client,
            allow_loopback_http,
        })
    }

    async fn execute(
        &self,
        request: http::Request<Vec<u8>>,
    ) -> Result<http::Response<Vec<u8>>, io::Error> {
        let (parts, body) = request.into_parts();
        if body.len() > MAX_OIDC_REQUEST_BYTES {
            return Err(io::Error::other("OIDC request body exceeded fixed limit"));
        }
        let url = reqwest::Url::parse(&parts.uri.to_string())
            .map_err(|error| io::Error::other(format!("invalid OIDC endpoint: {error}")))?;
        validate_remote_url(&url, self.allow_loopback_http, "discovered OIDC endpoint")
            .map_err(io::Error::other)?;

        let mut response = self
            .client
            .request(parts.method, url)
            .headers(parts.headers)
            .body(body)
            .send()
            .await
            .map_err(io::Error::other)?;
        if response
            .content_length()
            .is_some_and(|length| length > MAX_OIDC_RESPONSE_BYTES as u64)
        {
            return Err(io::Error::other("OIDC response body exceeded fixed limit"));
        }

        let status = response.status();
        let headers = response.headers().clone();
        let mut body = Vec::with_capacity(
            response
                .content_length()
                .map_or(0, |length| length.min(MAX_OIDC_RESPONSE_BYTES as u64) as usize),
        );
        while let Some(chunk) = response.chunk().await.map_err(io::Error::other)? {
            if body.len().saturating_add(chunk.len()) > MAX_OIDC_RESPONSE_BYTES {
                return Err(io::Error::other("OIDC response body exceeded fixed limit"));
            }
            body.extend_from_slice(&chunk);
        }

        let mut result = http::Response::builder().status(status);
        let result_headers = result
            .headers_mut()
            .ok_or_else(|| io::Error::other("could not construct OIDC response headers"))?;
        *result_headers = headers;
        result
            .body(body)
            .map_err(|error| io::Error::other(format!("could not construct OIDC response: {error}")))
    }
}

impl<'client> AsyncHttpClient<'client> for OidcHttpClient {
    type Error = io::Error;
    type Future = Pin<
        Box<
            dyn Future<Output = Result<http::Response<Vec<u8>>, Self::Error>>
                + Send
                + 'client,
        >,
    >;

    fn call(&'client self, request: http::Request<Vec<u8>>) -> Self::Future {
        Box::pin(self.execute(request))
    }
}
