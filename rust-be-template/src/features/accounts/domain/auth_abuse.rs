//! Domain types for bounded authentication-abuse controls.

use std::time::Duration;

/// Authentication surface with an independent abuse budget.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AuthEndpoint {
    Login,
    Signup,
    PasswordResetRequest,
    PasswordResetSubmit,
    EmailVerification,
    OidcStart,
}

impl AuthEndpoint {
    /// Stable low-cardinality name used in metrics and structured logs.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Login => "login",
            Self::Signup => "signup",
            Self::PasswordResetRequest => "password_reset_request",
            Self::PasswordResetSubmit => "password_reset_submit",
            Self::EmailVerification => "email_verification",
            Self::OidcStart => "oidc_start",
        }
    }

    /// Resolve the endpoint without retaining request paths in limiter state.
    pub fn from_path(path: &str) -> Option<Self> {
        match path {
            "/api/auth/login" => Some(Self::Login),
            "/api/auth/signup" => Some(Self::Signup),
            "/api/auth/reset-password-request" => Some(Self::PasswordResetRequest),
            "/api/auth/reset-password" => Some(Self::PasswordResetSubmit),
            "/api/auth/verify-user-email" => Some(Self::EmailVerification),
            "/api/auth/oidc/login/start" | "/api/auth/oidc/link/start" => {
                Some(Self::OidcStart)
            }
            _ => None,
        }
    }
}

/// Identity material accepted transiently for keyed hashing.
#[derive(Clone, Copy)]
pub enum AuthIdentity<'a> {
    Email(&'a str),
    UserName(&'a str),
    Token(&'a [u8]),
}

/// Low-cardinality throttle dimension safe to emit in logs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthThrottleDimension {
    IpPrefix,
    Email,
    UserName,
    Token,
}

impl AuthThrottleDimension {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::IpPrefix => "ip_prefix",
            Self::Email => "email_digest",
            Self::UserName => "user_name_digest",
            Self::Token => "token_digest",
        }
    }
}

/// A fail-closed admission rejection with enough information for `Retry-After`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthThrottleRejection {
    endpoint: AuthEndpoint,
    dimension: AuthThrottleDimension,
    retry_after: Duration,
    capacity_saturated: bool,
}

impl AuthThrottleRejection {
    pub(crate) const fn new(
        endpoint: AuthEndpoint,
        dimension: AuthThrottleDimension,
        retry_after: Duration,
        capacity_saturated: bool,
    ) -> Self {
        Self {
            endpoint,
            dimension,
            retry_after,
            capacity_saturated,
        }
    }

    pub const fn endpoint(self) -> AuthEndpoint {
        self.endpoint
    }

    pub const fn dimension(self) -> AuthThrottleDimension {
        self.dimension
    }

    pub const fn retry_after(self) -> Duration {
        self.retry_after
    }

    pub const fn capacity_saturated(self) -> bool {
        self.capacity_saturated
    }
}

/// Counts removed by one scheduled expiry sweep.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AuthAbusePruneReport {
    pub ip_records_removed: usize,
    pub identity_records_removed: usize,
    pub ip_records_retained: usize,
    pub identity_records_retained: usize,
}
