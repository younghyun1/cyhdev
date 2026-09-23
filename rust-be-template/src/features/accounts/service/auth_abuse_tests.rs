use std::{
    net::{IpAddr, Ipv4Addr},
    time::{Duration, Instant},
};

use zeroize::Zeroizing;

use super::auth_abuse::{AuthAbuseService, canonical_ip_prefix};
use crate::features::accounts::domain::auth_abuse::{
    AuthEndpoint, AuthIdentity, AuthThrottleDimension, FailureBudget,
};

fn service(capacity: usize) -> AuthAbuseService {
    AuthAbuseService::with_capacity(Zeroizing::new([7_u8; 32]), Some(capacity))
}

#[tokio::test]
async fn login_ip_windows_reject_the_eleventh_attempt() {
    let service = service(16);
    let ip = IpAddr::from([192, 0, 2, 1]);
    for _ in 0..10 {
        assert!(service.check_ip(AuthEndpoint::Login, ip).await.is_ok());
    }

    let rejection = service.check_ip(AuthEndpoint::Login, ip).await;
    assert!(
        rejection.is_err_and(|rejection| {
            rejection.dimension() == AuthThrottleDimension::IpPrefix
                && !rejection.capacity_saturated()
                && rejection.retry_after() <= Duration::from_secs(60)
        }),
        "login IP limit admitted an eleventh attempt"
    );
}

#[tokio::test]
async fn ipv6_addresses_share_a_slash_64_budget() {
    let service = service(16);
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

#[test]
fn ipv4_mapped_sources_keep_their_full_address() {
    let first = IpAddr::V6(Ipv4Addr::new(192, 0, 2, 1).to_ipv6_mapped());
    let second = IpAddr::V6(Ipv4Addr::new(198, 51, 100, 7).to_ipv6_mapped());
    assert_ne!(canonical_ip_prefix(first), canonical_ip_prefix(second));
    assert_eq!(
        canonical_ip_prefix(first),
        canonical_ip_prefix(IpAddr::from([192, 0, 2, 1]))
    );
}

#[tokio::test]
async fn login_failures_lock_only_the_email_from_that_source() {
    let service = service(16);
    let attacker = IpAddr::from([192, 0, 2, 1]);
    let owner = IpAddr::from([198, 51, 100, 7]);
    let email = "User@example.test";
    for _ in 0..4 {
        assert_eq!(
            service
                .record_failure(
                    AuthEndpoint::Login,
                    AuthIdentity::EmailFromIp(email, attacker)
                )
                .await,
            Ok(FailureBudget::Remaining)
        );
    }
    assert_eq!(
        service
            .record_failure(
                AuthEndpoint::Login,
                AuthIdentity::EmailFromIp(" user@EXAMPLE.test ", attacker)
            )
            .await,
        Ok(FailureBudget::Exhausted)
    );
    assert!(
        service
            .ensure_failure_budget(
                AuthEndpoint::Login,
                AuthIdentity::EmailFromIp(email, attacker)
            )
            .await
            .is_err()
    );
    assert!(
        service
            .ensure_failure_budget(AuthEndpoint::Login, AuthIdentity::EmailFromIp(email, owner))
            .await
            .is_ok()
    );
    service
        .forget_identity(
            AuthEndpoint::Login,
            AuthIdentity::EmailFromIp(email, attacker),
        )
        .await;
    assert!(
        service
            .ensure_failure_budget(
                AuthEndpoint::Login,
                AuthIdentity::EmailFromIp(email, attacker)
            )
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn budget_checks_do_not_consume_attempts() {
    let service = service(16);
    for _ in 0..100 {
        assert!(
            service
                .ensure_failure_budget(AuthEndpoint::Login, AuthIdentity::Email("a@example.test"))
                .await
                .is_ok()
        );
    }
}

#[tokio::test]
async fn normalized_email_variants_share_an_identity_budget() {
    let service = service(16);
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
                .check_identity(
                    AuthEndpoint::PasswordResetRequest,
                    AuthIdentity::Email(email)
                )
                .await
                .is_ok()
                == (email == "User@example.test")
        );
    }
}

#[tokio::test]
async fn full_tables_evict_single_attempt_keys_instead_of_rejecting_new_ones() {
    let service = service(2);
    let start = Instant::now();
    for last_octet in 1..=64 {
        let ip = IpAddr::from([192, 0, 2, last_octet]);
        assert!(
            service
                .check_ip_at(AuthEndpoint::Login, ip, start)
                .await
                .is_ok()
        );
    }
    let report = service
        .prune_expired_at(start + Duration::from_secs(3_601))
        .await;
    assert_eq!(report.ip_records_removed, 2);
}

#[tokio::test]
async fn endpoint_tables_are_independent() {
    let service = service(1);
    let now = Instant::now();
    let ip = IpAddr::from([192, 0, 2, 1]);
    assert!(
        service
            .check_ip_at(AuthEndpoint::Signup, ip, now)
            .await
            .is_ok()
    );
    assert!(
        service
            .check_ip_at(AuthEndpoint::Signup, ip, now)
            .await
            .is_ok()
    );
    // The signup table is now full of a multi-attempt record, yet login is unaffected.
    assert!(
        service
            .check_ip_at(AuthEndpoint::Signup, IpAddr::from([192, 0, 2, 2]), now)
            .await
            .is_err_and(|rejection| rejection.capacity_saturated())
    );
    assert!(
        service
            .check_ip_at(AuthEndpoint::Login, IpAddr::from([192, 0, 2, 2]), now)
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn unexpected_identity_kind_is_rejected_without_retention() {
    let service = service(16);
    let token = uuid::Uuid::from_u128(0x018f0a40c3197d318f506a4d4dd4f130);
    let rejection = service
        .check_identity(AuthEndpoint::Login, AuthIdentity::Token(token.as_bytes()))
        .await;
    assert!(
        rejection.is_err_and(|rejection| rejection.capacity_saturated()),
        "unexpected identity kind was admitted"
    );
}

#[tokio::test]
async fn fifth_wrong_confirmation_exhausts_the_account_budget() {
    let service = service(16);
    let account = AuthIdentity::Account(uuid::Uuid::from_u128(7));
    for _ in 0..4 {
        assert_eq!(
            service
                .record_failure(AuthEndpoint::PasswordConfirmation, account)
                .await,
            Ok(FailureBudget::Remaining)
        );
    }
    assert_eq!(
        service
            .record_failure(AuthEndpoint::PasswordConfirmation, account)
            .await,
        Ok(FailureBudget::Exhausted)
    );
    assert!(
        service
            .ensure_failure_budget(AuthEndpoint::PasswordConfirmation, account)
            .await
            .is_err()
    );
    let ip = IpAddr::from([192, 0, 2, 9]);
    for _ in 0..10 {
        assert!(
            service
                .check_ip(AuthEndpoint::PasswordConfirmation, ip)
                .await
                .is_ok()
        );
    }
    assert!(
        service
            .check_ip(AuthEndpoint::PasswordConfirmation, ip)
            .await
            .is_err()
    );
}
