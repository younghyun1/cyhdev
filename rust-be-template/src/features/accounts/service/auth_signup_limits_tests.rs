//! Signup retry allowances retain independent hourly, daily, and identity bounds.

use std::{
    net::IpAddr,
    time::{Duration, Instant},
};

use zeroize::Zeroizing;

use super::auth_abuse::AuthAbuseService;
use crate::features::accounts::domain::auth_abuse::{
    AuthEndpoint, AuthIdentity, AuthThrottleDimension,
};

#[tokio::test]
async fn signup_ip_allows_ten_per_hour_and_twenty_per_day() {
    let service = AuthAbuseService::with_capacity(Zeroizing::new([7_u8; 32]), Some(16));
    let ip = IpAddr::from([192, 0, 2, 1]);
    let start = Instant::now();
    let hour = Duration::from_secs(3_600);
    let day = Duration::from_secs(86_400);

    for now in [start, start + hour] {
        for _ in 0..10 {
            assert!(
                service
                    .check_ip_at(AuthEndpoint::Signup, ip, now)
                    .await
                    .is_ok()
            );
        }
        assert!(
            service
                .check_ip_at(AuthEndpoint::Signup, ip, now)
                .await
                .is_err()
        );
    }

    // Expiring the hourly window must not forgive the exhausted daily allowance.
    assert!(
        service
            .check_ip_at(AuthEndpoint::Signup, ip, start + hour * 2)
            .await
            .is_err_and(|rejection| {
                rejection.dimension() == AuthThrottleDimension::IpPrefix
                    && !rejection.capacity_saturated()
                    && rejection.retry_after() == day - hour * 2
            })
    );
    assert!(
        service
            .check_ip_at(AuthEndpoint::Signup, ip, start + day)
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn signup_email_and_username_each_allow_five_normalized_attempts() {
    let service = AuthAbuseService::with_capacity(Zeroizing::new([7_u8; 32]), Some(16));
    for _ in 0..5 {
        assert!(
            service
                .check_identity(
                    AuthEndpoint::Signup,
                    AuthIdentity::Email("User@example.test")
                )
                .await
                .is_ok()
        );
        assert!(
            service
                .check_identity(AuthEndpoint::Signup, AuthIdentity::UserName("ExampleUser"))
                .await
                .is_ok()
        );
    }

    for (identity, dimension) in [
        (
            AuthIdentity::Email(" user@EXAMPLE.test "),
            AuthThrottleDimension::Email,
        ),
        (
            AuthIdentity::UserName(" exampleUSER "),
            AuthThrottleDimension::UserName,
        ),
    ] {
        assert!(
            service
                .check_identity(AuthEndpoint::Signup, identity)
                .await
                .is_err_and(|rejection| {
                    rejection.dimension() == dimension
                        && !rejection.capacity_saturated()
                        && rejection.retry_after() > Duration::from_secs(86_000)
                        && rejection.retry_after() <= Duration::from_secs(86_400)
                })
        );
    }

    assert!(
        service
            .check_identity(
                AuthEndpoint::Signup,
                AuthIdentity::Email("other@example.test")
            )
            .await
            .is_ok()
    );
    assert!(
        service
            .check_identity(AuthEndpoint::Signup, AuthIdentity::UserName("OtherUser"))
            .await
            .is_ok()
    );
}
