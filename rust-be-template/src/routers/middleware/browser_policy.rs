//! Browser security headers applied to every response.
//!
//! The application policy is built once at startup from configuration: the public
//! origin, the media bucket origin, and hashes of the inline scripts in the embedded
//! application shell. Responses that already carry a stricter policy keep it, such as
//! the sandbox on uploaded WASM demos and `no-referrer` on authentication routes.
//!
//! Third-party applications served from this origin (the squaremap web map and the
//! EU5 browser app) get a response-enforced CSP sandbox without `allow-same-origin`,
//! so their scripts run in an opaque origin even on direct navigation and cannot call
//! the API as the viewer. Their static files carry `Access-Control-Allow-Origin: *`
//! without credentials, because an opaque-origin document fetches module scripts,
//! WebAssembly, JSON, and fonts in CORS mode.

use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::{HeaderMap, HeaderName, HeaderValue, header},
    middleware::Next,
    response::Response,
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use sha2::{Digest, Sha256};

const PERMISSIONS_POLICY: HeaderName = HeaderName::from_static("permissions-policy");
/// Camera and microphone serve calls on this origin; nothing else is delegated.
const PERMISSIONS: &str = "camera=(self), microphone=(self), display-capture=(), geolocation=(), payment=(), usb=(), serial=(), hid=(), midi=()";
/// One year, without `includeSubDomains`: other hosts under the domain are not
/// known to serve HTTPS, and a subdomain pin cannot be undone for a year.
const STRICT_TRANSPORT_SECURITY: &str = "max-age=31536000";
const REFERRER_POLICY: &str = "strict-origin-when-cross-origin";
/// Nominatim place search used by the batch-upload location picker.
const GEOCODER_ORIGIN: &str = "https://nominatim.openstreetmap.org";
/// Hashes beyond this many inline scripts indicate a broken shell, not a policy need.
const MAX_INLINE_SCRIPTS: usize = 16;

/// A same-origin application isolated in an opaque-origin sandbox.
struct EmbeddedApp {
    /// Exact path of the directory without its trailing slash; children follow it.
    root: &'static str,
    policy: &'static str,
}

/// Sandbox flags mirror each iframe's `sandbox` attribute; both apply together.
const EMBEDDED_APPS: [EmbeddedApp; 2] = [
    EmbeddedApp {
        root: "/minecraft/map",
        policy: "sandbox allow-scripts allow-popups allow-popups-to-escape-sandbox; frame-ancestors 'self'",
    },
    EmbeddedApp {
        root: "/eu5-locations-db/app",
        policy: "sandbox allow-scripts allow-popups allow-popups-to-escape-sandbox allow-top-navigation-by-user-activation; frame-ancestors 'self'",
    },
];

/// Returns the sandbox policy when `path` belongs to an embedded application.
pub fn embedded_app_policy(path: &str) -> Option<&'static str> {
    EMBEDDED_APPS.iter().find_map(|app| {
        let inside = path == app.root
            || path
                .strip_prefix(app.root)
                .is_some_and(|rest| rest.starts_with('/'));
        inside.then_some(app.policy)
    })
}

/// Startup inputs for the application Content-Security-Policy.
pub struct BrowserPolicyConfig {
    /// Canonical origin such as `https://cyhdev.com`; its WebSocket form is derived.
    pub app_origin: String,
    /// Origins serving uploaded images.
    pub media_origins: Vec<String>,
    /// `'sha256-…'` sources for inline scripts in the application shell.
    pub inline_script_hashes: Vec<String>,
    /// Sends HSTS; disabled for loopback development so `localhost` is not pinned.
    pub strict_transport_security: bool,
}

/// Precomputed header values shared by every request.
pub struct BrowserSecurityPolicy {
    application_csp: HeaderValue,
    strict_transport_security: bool,
}

impl BrowserSecurityPolicy {
    pub fn new(config: &BrowserPolicyConfig) -> anyhow::Result<Self> {
        let policy = application_content_security_policy(config)?;
        Ok(Self {
            application_csp: HeaderValue::from_str(&policy)
                .map_err(|error| anyhow::anyhow!("invalid Content-Security-Policy: {error}"))?,
            strict_transport_security: config.strict_transport_security,
        })
    }

    fn apply(&self, embedded_policy: Option<&'static str>, headers: &mut HeaderMap) {
        match embedded_policy {
            Some(policy) => {
                headers.insert(
                    header::CONTENT_SECURITY_POLICY,
                    HeaderValue::from_static(policy),
                );
                headers.insert(
                    header::ACCESS_CONTROL_ALLOW_ORIGIN,
                    HeaderValue::from_static("*"),
                );
            }
            None => {
                if !headers.contains_key(header::CONTENT_SECURITY_POLICY) {
                    headers.insert(
                        header::CONTENT_SECURITY_POLICY,
                        self.application_csp.clone(),
                    );
                }
            }
        }
        insert_if_absent(headers, header::X_CONTENT_TYPE_OPTIONS, "nosniff");
        insert_if_absent(headers, header::REFERRER_POLICY, REFERRER_POLICY);
        insert_if_absent(headers, PERMISSIONS_POLICY, PERMISSIONS);
        if self.strict_transport_security {
            insert_if_absent(
                headers,
                header::STRICT_TRANSPORT_SECURITY,
                STRICT_TRANSPORT_SECURITY,
            );
        }
    }
}

fn insert_if_absent(headers: &mut HeaderMap, name: HeaderName, value: &'static str) {
    if !headers.contains_key(&name) {
        headers.insert(name, HeaderValue::from_static(value));
    }
}

/// Applies [`BrowserSecurityPolicy`] after the rest of the stack has produced a response.
pub async fn apply_browser_security_headers(
    State(policy): State<Arc<BrowserSecurityPolicy>>,
    request: Request,
    next: Next,
) -> Response {
    let embedded_policy = embedded_app_policy(request.uri().path());
    let mut response = next.run(request).await;
    policy.apply(embedded_policy, response.headers_mut());
    response
}

/// Builds the application policy from the usage the frontend actually has.
///
/// `style-src` allows inline styles because the Markdown editor, Leaflet, and chart
/// tooltips set `style` attributes at runtime. No page on this origin instantiates
/// WebAssembly (demos and the EU5 app run in sandboxed frames with their own policy),
/// so `'wasm-unsafe-eval'` is absent.
pub fn application_content_security_policy(config: &BrowserPolicyConfig) -> anyhow::Result<String> {
    let websocket_origin = websocket_origin(&config.app_origin)?;
    let mut script = vec!["'self'".to_owned()];
    script.extend(config.inline_script_hashes.iter().cloned());
    let mut image = vec!["'self'", "data:", "blob:"]
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    image.extend(config.media_origins.iter().cloned());
    // Blog Markdown may embed images from any HTTPS host, and map tiles come from
    // OpenStreetMap; images cannot execute script, so any HTTPS source is allowed.
    image.push("https:".to_owned());
    let directives = [
        ("default-src", vec!["'self'".to_owned()]),
        ("script-src", script),
        (
            "style-src",
            vec!["'self'".to_owned(), "'unsafe-inline'".to_owned()],
        ),
        ("img-src", image),
        ("font-src", vec!["'self'".to_owned(), "data:".to_owned()]),
        (
            "connect-src",
            vec![
                "'self'".to_owned(),
                websocket_origin,
                GEOCODER_ORIGIN.to_owned(),
            ],
        ),
        ("frame-src", vec!["'self'".to_owned()]),
        ("object-src", vec!["'none'".to_owned()]),
        ("base-uri", vec!["'self'".to_owned()]),
        ("form-action", vec!["'self'".to_owned()]),
        ("frame-ancestors", vec!["'self'".to_owned()]),
    ];
    let mut policy = Vec::with_capacity(directives.len());
    for (name, sources) in directives {
        for source in &sources {
            validate_source(source)?;
        }
        policy.push(format!("{name} {}", sources.join(" ")));
    }
    Ok(policy.join("; "))
}

/// Same-origin WebSockets use `ws:`/`wss:`, which older engines do not match with `'self'`.
fn websocket_origin(app_origin: &str) -> anyhow::Result<String> {
    if let Some(authority) = app_origin.strip_prefix("https://") {
        Ok(format!("wss://{authority}"))
    } else if let Some(authority) = app_origin.strip_prefix("http://") {
        Ok(format!("ws://{authority}"))
    } else {
        Err(anyhow::anyhow!("application origin must be HTTP or HTTPS"))
    }
}

/// Rejects characters that would end a source list or directive early.
fn validate_source(source: &str) -> anyhow::Result<()> {
    let valid = !source.is_empty()
        && source
            .bytes()
            .all(|byte| byte.is_ascii_graphic() && !matches!(byte, b';' | b','));
    if valid {
        Ok(())
    } else {
        Err(anyhow::anyhow!("invalid CSP source expression {source:?}"))
    }
}

/// Returns `'sha256-…'` sources for each inline `<script>` in `html`.
///
/// The HTML parser normalizes CRLF and lone CR to LF before a script's text reaches
/// the CSP check, so hashing applies the same normalization to match on any checkout.
pub fn inline_script_hashes(html: &str) -> Vec<String> {
    let lower = html.to_ascii_lowercase();
    let mut hashes = Vec::new();
    let mut cursor = 0;
    while let Some(offset) = lower[cursor..].find("<script") {
        let tag_start = cursor + offset;
        let Some(tag_length) = lower[tag_start..].find('>') else {
            break;
        };
        let body_start = tag_start + tag_length + 1;
        let Some(body_length) = lower[body_start..].find("</script") else {
            break;
        };
        let tag = &lower[tag_start + "<script".len()..body_start - 1];
        let external = tag
            .split(|character: char| character.is_ascii_whitespace())
            .any(|attribute| attribute == "src" || attribute.starts_with("src="));
        if !external && hashes.len() < MAX_INLINE_SCRIPTS {
            let body = html[body_start..body_start + body_length]
                .replace("\r\n", "\n")
                .replace('\r', "\n");
            hashes.push(format!(
                "'sha256-{}'",
                STANDARD.encode(Sha256::digest(body))
            ));
        }
        cursor = body_start + body_length;
    }
    hashes
}

#[cfg(test)]
#[path = "browser_policy_tests.rs"]
mod tests;
