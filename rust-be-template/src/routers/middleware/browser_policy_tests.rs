use axum::http::{HeaderMap, HeaderValue, header};
use serde_json::Value;

use super::{
    BrowserPolicyConfig, BrowserSecurityPolicy, PERMISSIONS, PERMISSIONS_POLICY,
    application_content_security_policy, embedded_app_policy, inline_script_hashes,
};

/// Shared with the browser check in `solid-csr-spa-template/e2e/security-headers.spec.ts`.
const PREVIEW_FIXTURE: &str =
    include_str!("../../../../solid-csr-spa-template/e2e/security-headers.json");
const APPLICATION_SHELL: &str = include_str!("../../../../solid-csr-spa-template/index.html");

fn production_config() -> BrowserPolicyConfig {
    BrowserPolicyConfig {
        app_origin: "https://cyhdev.com".to_owned(),
        media_origins: vec!["https://cyhdev-img.s3.us-west-1.amazonaws.com".to_owned()],
        inline_script_hashes: vec!["'sha256-abc='".to_owned()],
        strict_transport_security: true,
    }
}

#[test]
fn application_policy_lists_only_used_sources() -> anyhow::Result<()> {
    assert_eq!(
        application_content_security_policy(&production_config())?,
        "default-src 'self'; script-src 'self' 'sha256-abc='; style-src 'self' 'unsafe-inline'; \
         img-src 'self' data: blob: https://cyhdev-img.s3.us-west-1.amazonaws.com https:; \
         font-src 'self' data:; connect-src 'self' wss://cyhdev.com https://nominatim.openstreetmap.org; \
         frame-src 'self'; object-src 'none'; base-uri 'self'; form-action 'self'; frame-ancestors 'self'"
    );
    Ok(())
}

#[test]
fn configuration_cannot_inject_directives() {
    let mut config = production_config();
    config.media_origins = vec!["https://evil.example; script-src *".to_owned()];
    assert!(application_content_security_policy(&config).is_err());
    let mut config = production_config();
    config.app_origin = "ftp://cyhdev.com".to_owned();
    assert!(application_content_security_policy(&config).is_err());
}

/// The browser check applies the fixture's headers to the built frontend; this keeps the
/// fixture identical to what the backend sends for the same configuration.
#[test]
fn browser_check_fixture_matches_the_backend_policy() -> anyhow::Result<()> {
    let fixture: Value = serde_json::from_str(PREVIEW_FIXTURE)?;
    let text = |key: &str| fixture[key].as_str().map(str::to_owned).unwrap_or_default();
    let media_origins = fixture["mediaOrigins"]
        .as_array()
        .map(|origins| {
            origins
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();
    let config = BrowserPolicyConfig {
        app_origin: text("origin"),
        media_origins,
        inline_script_hashes: inline_script_hashes(APPLICATION_SHELL),
        strict_transport_security: false,
    };
    let expected = application_content_security_policy(&config)?;
    assert_eq!(
        text("contentSecurityPolicy"),
        expected,
        "update solid-csr-spa-template/e2e/security-headers.json to the expected policy"
    );
    assert_eq!(text("permissionsPolicy"), PERMISSIONS);
    assert_eq!(
        Some(text("mapPolicy").as_str()),
        embedded_app_policy("/minecraft/map/")
    );
    assert_eq!(
        Some(text("eu5Policy").as_str()),
        embedded_app_policy("/eu5-locations-db/app/index.html")
    );
    Ok(())
}

#[test]
fn inline_scripts_are_hashed_and_external_scripts_skipped() {
    let html = "<head><SCRIPT>alert(1)</SCRIPT>\
                <script type=\"module\" src=\"/assets/app.js\"></script>\
                <script type=module>\r\nlet a = 1;\r\n</script></head>";
    assert_eq!(
        inline_script_hashes(html),
        vec![
            "'sha256-bhHHL3z2vDgxUt0W3dWQOrprscmda2Y5pLsLg4GF+pI='".to_owned(),
            "'sha256-wCpX6PRaV2tYabHPuD0EVrhGyeiYQpOhQOJ8OhOtDwA='".to_owned(),
        ]
    );
    assert!(inline_script_hashes("<script>unterminated").is_empty());
    assert_eq!(inline_script_hashes(APPLICATION_SHELL).len(), 1);
}

#[test]
fn embedded_paths_match_whole_segments_only() {
    for path in [
        "/minecraft/map",
        "/minecraft/map/",
        "/minecraft/map/tiles/world/0/0_0.png",
        "/eu5-locations-db/app/index.html",
        "/eu5-locations-db/app/pkg/eu5_location_filter_bg.wasm",
    ] {
        assert!(embedded_app_policy(path).is_some(), "{path}");
    }
    for path in [
        "/minecraft",
        "/minecraft/mapx",
        "/eu5-locations-db",
        "/eu5-locations-db/apps",
    ] {
        assert!(embedded_app_policy(path).is_none(), "{path}");
    }
}

#[test]
fn application_headers_fill_gaps_without_weakening_route_policies() -> anyhow::Result<()> {
    let policy = BrowserSecurityPolicy::new(&production_config())?;
    let mut headers = HeaderMap::new();
    policy.apply(None, &mut headers);
    assert!(
        headers[header::CONTENT_SECURITY_POLICY]
            .to_str()
            .is_ok_and(|value| value.starts_with("default-src 'self'"))
    );
    assert_eq!(headers[header::X_CONTENT_TYPE_OPTIONS], "nosniff");
    assert_eq!(
        headers[header::REFERRER_POLICY],
        "strict-origin-when-cross-origin"
    );
    assert_eq!(
        headers[header::STRICT_TRANSPORT_SECURITY],
        "max-age=31536000"
    );
    assert_eq!(headers[PERMISSIONS_POLICY], PERMISSIONS);
    assert!(!headers.contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN));

    // Uploaded demo HTML and authentication responses keep their stricter values.
    let mut stricter = HeaderMap::new();
    stricter.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("sandbox allow-scripts"),
    );
    stricter.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    policy.apply(None, &mut stricter);
    assert_eq!(
        stricter[header::CONTENT_SECURITY_POLICY],
        "sandbox allow-scripts"
    );
    assert_eq!(stricter[header::REFERRER_POLICY], "no-referrer");
    Ok(())
}

#[test]
fn embedded_apps_get_an_opaque_origin_sandbox_and_public_cors() -> anyhow::Result<()> {
    let mut config = production_config();
    config.strict_transport_security = false;
    let policy = BrowserSecurityPolicy::new(&config)?;
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src *"),
    );
    policy.apply(embedded_app_policy("/minecraft/map/"), &mut headers);
    let csp = headers[header::CONTENT_SECURITY_POLICY]
        .to_str()
        .unwrap_or_default();
    assert!(csp.starts_with("sandbox allow-scripts"));
    assert!(!csp.contains("allow-same-origin"));
    assert_eq!(headers[header::ACCESS_CONTROL_ALLOW_ORIGIN], "*");
    assert!(!headers.contains_key(header::ACCESS_CONTROL_ALLOW_CREDENTIALS));
    assert!(!headers.contains_key(header::STRICT_TRANSPORT_SECURITY));
    Ok(())
}
