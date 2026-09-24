//! Static authentication throttle policies and per-table capacities.

use std::time::Duration;

use crate::features::accounts::domain::auth_abuse::{AuthEndpoint, AuthThrottleDimension};

const MINUTE: Duration = Duration::from_secs(60);
const FIFTEEN_MINUTES: Duration = Duration::from_secs(15 * 60);
const HOUR: Duration = Duration::from_secs(60 * 60);
const DAY: Duration = Duration::from_secs(24 * 60 * 60);

/// Upper bound on windows in one policy; each table record stores this many counters.
pub(super) const MAX_WINDOWS_PER_POLICY: usize = 2;

const LOGIN_IP: [FixedWindowLimit; 2] = [limit(10, MINUTE), limit(50, HOUR)];
/// Failed logins for one email from one source: the strict lockout a guesser hits first.
const LOGIN_EMAIL_FROM_IP_FAILURES: [FixedWindowLimit; 1] = [limit(5, FIFTEEN_MINUTES)];
/// Failed logins for one email from every source: a looser cap on distributed guessing.
const LOGIN_EMAIL_FAILURES: [FixedWindowLimit; 1] = [limit(20, HOUR)];
const SIGNUP_IP: [FixedWindowLimit; 2] = [limit(10, HOUR), limit(20, DAY)];
const SIGNUP_IDENTITY: [FixedWindowLimit; 1] = [limit(5, DAY)];
const RESET_REQUEST_IP: [FixedWindowLimit; 2] = [limit(5, HOUR), limit(20, DAY)];
const RESET_REQUEST_IDENTITY: [FixedWindowLimit; 2] = [limit(1, FIFTEEN_MINUTES), limit(3, DAY)];
const RESET_SUBMIT_IP: [FixedWindowLimit; 1] = [limit(10, FIFTEEN_MINUTES)];
const RESET_SUBMIT_TOKEN: [FixedWindowLimit; 1] = [limit(5, FIFTEEN_MINUTES)];
const VERIFY_IP: [FixedWindowLimit; 1] = [limit(20, HOUR)];
const VERIFY_TOKEN: [FixedWindowLimit; 1] = [limit(5, HOUR)];
const OIDC_START_IP: [FixedWindowLimit; 2] = [limit(10, MINUTE), limit(50, HOUR)];
/// Password confirmations from one source, counted per attempt before Argon2 work.
const CONFIRMATION_IP: [FixedWindowLimit; 2] = [limit(10, FIFTEEN_MINUTES), limit(50, DAY)];
/// Wrong confirmations for one account; exhausting it revokes the presenting session.
const CONFIRMATION_ACCOUNT_FAILURES: [FixedWindowLimit; 1] = [limit(5, HOUR)];

#[derive(Clone, Copy, Debug)]
pub(super) struct FixedWindowLimit {
    pub(super) attempts: u32,
    pub(super) duration: Duration,
}

const fn limit(attempts: u32, duration: Duration) -> FixedWindowLimit {
    FixedWindowLimit { attempts, duration }
}

/// One independently bounded table: a flood against one endpoint class cannot evict or
/// saturate the budgets of another.
#[derive(Clone, Copy, Debug)]
pub(super) struct ThrottleTable {
    pub(super) endpoint: AuthEndpoint,
    pub(super) dimension: AuthThrottleDimension,
    pub(super) capacity: usize,
    pub(super) limits: &'static [FixedWindowLimit],
}

const fn table(
    endpoint: AuthEndpoint,
    dimension: AuthThrottleDimension,
    capacity: usize,
    limits: &'static [FixedWindowLimit],
) -> ThrottleTable {
    ThrottleTable {
        endpoint,
        dimension,
        capacity,
        limits,
    }
}

/// Every throttle table. Capacities total 73,728 records of roughly 150 bytes each, so the
/// limiter stays near 11 MiB at worst.
pub(super) const THROTTLE_TABLES: [ThrottleTable; 15] = {
    use AuthEndpoint as E;
    use AuthThrottleDimension as D;
    [
        table(E::Login, D::IpPrefix, 8_192, &LOGIN_IP),
        table(
            E::Login,
            D::EmailAndIp,
            8_192,
            &LOGIN_EMAIL_FROM_IP_FAILURES,
        ),
        table(E::Login, D::Email, 8_192, &LOGIN_EMAIL_FAILURES),
        table(E::Signup, D::IpPrefix, 4_096, &SIGNUP_IP),
        table(E::Signup, D::Email, 4_096, &SIGNUP_IDENTITY),
        table(E::Signup, D::UserName, 4_096, &SIGNUP_IDENTITY),
        table(
            E::PasswordResetRequest,
            D::IpPrefix,
            4_096,
            &RESET_REQUEST_IP,
        ),
        table(
            E::PasswordResetRequest,
            D::Email,
            4_096,
            &RESET_REQUEST_IDENTITY,
        ),
        table(E::PasswordResetSubmit, D::IpPrefix, 4_096, &RESET_SUBMIT_IP),
        table(E::PasswordResetSubmit, D::Token, 4_096, &RESET_SUBMIT_TOKEN),
        table(E::EmailVerification, D::IpPrefix, 4_096, &VERIFY_IP),
        table(E::EmailVerification, D::Token, 4_096, &VERIFY_TOKEN),
        table(E::OidcStart, D::IpPrefix, 4_096, &OIDC_START_IP),
        table(
            E::PasswordConfirmation,
            D::IpPrefix,
            4_096,
            &CONFIRMATION_IP,
        ),
        table(
            E::PasswordConfirmation,
            D::Account,
            4_096,
            &CONFIRMATION_ACCOUNT_FAILURES,
        ),
    ]
};

/// Index of the table for an endpoint and dimension, when that pair is throttled.
pub(super) fn table_index(
    endpoint: AuthEndpoint,
    dimension: AuthThrottleDimension,
) -> Option<usize> {
    THROTTLE_TABLES
        .iter()
        .position(|table| table.endpoint == endpoint && table.dimension == dimension)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_policy_fits_the_record_layout_and_tables_are_unique() {
        for (index, table) in THROTTLE_TABLES.iter().enumerate() {
            assert!(!table.limits.is_empty());
            assert!(table.limits.len() <= MAX_WINDOWS_PER_POLICY);
            assert!(table.capacity > 0);
            assert!(
                THROTTLE_TABLES[index + 1..]
                    .iter()
                    .all(|other| other.endpoint != table.endpoint
                        || other.dimension != table.dimension)
            );
        }
    }
}
