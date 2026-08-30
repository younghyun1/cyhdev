//! Static authentication throttle policies.

use std::time::Duration;

use crate::features::accounts::domain::auth_abuse::{AuthEndpoint, AuthThrottleDimension};

const MINUTE: Duration = Duration::from_secs(60);
const FIFTEEN_MINUTES: Duration = Duration::from_secs(15 * 60);
const HOUR: Duration = Duration::from_secs(60 * 60);
const DAY: Duration = Duration::from_secs(24 * 60 * 60);

const LOGIN_IP: [FixedWindowLimit; 2] = [limit(10, MINUTE), limit(50, HOUR)];
const LOGIN_IDENTITY: [FixedWindowLimit; 1] = [limit(5, FIFTEEN_MINUTES)];
const SIGNUP_IP: [FixedWindowLimit; 2] = [limit(3, HOUR), limit(10, DAY)];
const SIGNUP_IDENTITY: [FixedWindowLimit; 1] = [limit(2, DAY)];
const RESET_REQUEST_IP: [FixedWindowLimit; 2] = [limit(5, HOUR), limit(20, DAY)];
const RESET_REQUEST_IDENTITY: [FixedWindowLimit; 2] = [limit(1, FIFTEEN_MINUTES), limit(3, DAY)];
const RESET_SUBMIT_IP: [FixedWindowLimit; 1] = [limit(10, FIFTEEN_MINUTES)];
const RESET_SUBMIT_TOKEN: [FixedWindowLimit; 1] = [limit(5, FIFTEEN_MINUTES)];
const VERIFY_IP: [FixedWindowLimit; 1] = [limit(20, HOUR)];
const VERIFY_TOKEN: [FixedWindowLimit; 1] = [limit(5, HOUR)];
const OIDC_START_IP: [FixedWindowLimit; 2] = [limit(10, MINUTE), limit(50, HOUR)];

#[derive(Clone, Copy, Debug)]
pub(super) struct FixedWindowLimit {
    pub(super) attempts: u32,
    pub(super) duration: Duration,
}

const fn limit(attempts: u32, duration: Duration) -> FixedWindowLimit {
    FixedWindowLimit { attempts, duration }
}

pub(super) fn ip_limits(endpoint: AuthEndpoint) -> &'static [FixedWindowLimit] {
    match endpoint {
        AuthEndpoint::Login => &LOGIN_IP,
        AuthEndpoint::Signup => &SIGNUP_IP,
        AuthEndpoint::PasswordResetRequest => &RESET_REQUEST_IP,
        AuthEndpoint::PasswordResetSubmit => &RESET_SUBMIT_IP,
        AuthEndpoint::EmailVerification => &VERIFY_IP,
        AuthEndpoint::OidcStart => &OIDC_START_IP,
    }
}

pub(super) fn identity_limits(
    endpoint: AuthEndpoint,
    dimension: AuthThrottleDimension,
) -> &'static [FixedWindowLimit] {
    match (endpoint, dimension) {
        (AuthEndpoint::Login, AuthThrottleDimension::Email) => &LOGIN_IDENTITY,
        (AuthEndpoint::Signup, AuthThrottleDimension::Email | AuthThrottleDimension::UserName) => {
            &SIGNUP_IDENTITY
        }
        (AuthEndpoint::PasswordResetRequest, AuthThrottleDimension::Email) => {
            &RESET_REQUEST_IDENTITY
        }
        (AuthEndpoint::PasswordResetSubmit, AuthThrottleDimension::Token) => &RESET_SUBMIT_TOKEN,
        (AuthEndpoint::EmailVerification, AuthThrottleDimension::Token) => &VERIFY_TOKEN,
        _ => &[],
    }
}
