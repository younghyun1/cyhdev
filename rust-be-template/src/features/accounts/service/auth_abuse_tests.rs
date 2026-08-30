use std::{
    net::IpAddr,
    time::{Duration, Instant},
};

use zeroize::Zeroizing;

use super::auth_abuse::AuthAbuseService;
use crate::features::accounts::domain::auth_abuse::{
    AuthEndpoint, AuthIdentity, AuthThrottleDimension,
};

fn service(ip_capacity: usize, identity_capacity: usize) -> AuthAbuseService {
    AuthAbuseService::with_limits(Zeroizing::new([7_u8; 32]), ip_capacity, identity_capacity)
}

#[tokio::test]
async fn login_ip_windows_reject_the_eleventh_attempt() {
    let service = service(16, 16);
    let ip = IpAddr::from([192, 0, 2, 1]);
    for _ in 0..10 {
        assert!(service.check_ip(AuthEndpoint::Login, ip).await.is_ok());
    }

    let rejection = service.check_ip(AuthEndpoint::Login, ip).await;
    match rejection {
        Err(rejection) => {
            assert_eq!(rejection.dimension(), AuthThrottleDimension::IpPrefix);
            assert!(!rejection.capacity_saturated());
            assert!(rejection.retry_after() <= Duration::from_secs(60));
        }
        Ok(()) => panic!("login IP limit admitted an eleventh attempt"),
    }
}

#[tokio::test]
async fn ipv6_addresses_share_a_slash_64_budget() {
    let service = service(16, 16);
    for suffix in 1_u128..=10 {
        let ip = IpAddr::V6((0x20010db800000001_u128 << 64 | suffix).into());
        assert!(service.check_ip(AuthEndpoint::Login, ip).await.is_ok());
    }
    let same_prefix = IpAddr::V6((0x20010db800000001_u128 << 64 | 99).into());
    assert!(
        service
            .check_ip(AuthEndpoint::Login, same_prefix)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn normalized_email_variants_share_an_identity_budget() {
    let service = service(16, 16);
    let variants = [
        "User@example.test",
        " user@example.test ",
        "USER@EXAMPLE.TEST",
        "user@example.test",
        "User@Example.Test",
    ];
    for email in variants {
        assert!(
            service
                .check_identity(AuthEndpoint::Login, AuthIdentity::Email(email))
                .await
                .is_ok()
        );
    }
    assert!(
        service
            .check_identity(
                AuthEndpoint::Login,
                AuthIdentity::Email("user@example.test"),
            )
            .await
            .is_err()
    );
}

#[tokio::test]
async fn strict_capacity_fails_closed_until_expiry_cleanup() {
    let service = service(2, 2);
    let start = Instant::now();
    let first = IpAddr::from([192, 0, 2, 1]);
    let second = IpAddr::from([192, 0, 2, 2]);
    assert!(
        service
            .check_ip_at(AuthEndpoint::Login, first, start)
            .await
            .is_ok()
    );

    let rejection = service
        .check_ip_at(AuthEndpoint::Login, second, start)
        .await;
    match rejection {
        Err(rejection) => assert!(rejection.capacity_saturated()),
        Ok(()) => panic!("strict limiter capacity admitted another IP"),
    }

    let report = service
        .prune_expired_at(start + Duration::from_secs(3_601))
        .await;
    assert_eq!(report.ip_records_removed, 2);
    assert!(
        service
            .check_ip_at(
                AuthEndpoint::Login,
                second,
                start + Duration::from_secs(3_601),
            )
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn unexpected_identity_kind_is_rejected_without_retention() {
    let service = service(16, 16);
    let token = uuid::Uuid::from_u128(0x018f0a40c3197d318f506a4d4dd4f130);
    let rejection = service
        .check_identity(AuthEndpoint::Login, AuthIdentity::Token(token.as_bytes()))
        .await;
    match rejection {
        Err(rejection) => assert!(rejection.capacity_saturated()),
        Ok(()) => panic!("unexpected identity kind was admitted"),
    }
}
