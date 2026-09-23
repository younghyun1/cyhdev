//! Outermost layers shared by the API, static assets, and embedded applications.

use std::sync::Arc;

use axum::{
    Router,
    middleware::{from_fn, from_fn_with_state},
};

use super::static_assets::EmbeddedAssets;
use crate::{
    init::state::{DeploymentEnvironment, ServerState},
    routers::middleware::{
        browser_policy::{
            BrowserPolicyConfig, BrowserSecurityPolicy, apply_browser_security_headers,
            inline_script_hashes,
        },
        request_deadline::enforce_request_deadline,
    },
    util::s3::AWS_S3_BUCKET_NAME,
};

/// Wraps the complete router so every route, including fallbacks, gets the policies.
///
/// Security headers are outermost so deadline responses carry them too.
pub(super) fn apply(router: Router, state: &ServerState) -> anyhow::Result<Router> {
    let policy = Arc::new(BrowserSecurityPolicy::new(&browser_policy_config(state))?);
    Ok(router
        .layer(from_fn(enforce_request_deadline))
        .layer(from_fn_with_state(policy, apply_browser_security_headers)))
}

fn browser_policy_config(state: &ServerState) -> BrowserPolicyConfig {
    let region = state.media_region();
    // Stored links use the regional host; the global host is accepted for older links.
    let media_origins = vec![
        format!("https://{AWS_S3_BUCKET_NAME}.s3.{region}.amazonaws.com"),
        format!("https://{AWS_S3_BUCKET_NAME}.s3.amazonaws.com"),
    ];
    // Hash the shell actually embedded in this binary, so the policy always matches it.
    let inline_script_hashes = match EmbeddedAssets::get("index.html") {
        Some(shell) => inline_script_hashes(&String::from_utf8_lossy(&shell.data)),
        None => {
            tracing::warn!("No embedded index.html; the CSP allows no inline scripts");
            Vec::new()
        }
    };
    BrowserPolicyConfig {
        app_origin: state.public_app_origin().as_str().to_owned(),
        media_origins,
        inline_script_hashes,
        strict_transport_security: !matches!(
            state.get_deployment_environment(),
            DeploymentEnvironment::Local
        ),
    }
}
