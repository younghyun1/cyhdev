use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    time::Duration,
};

use super::{RequestLimitInputs, select, shared_policy, validate_database, validate_origin};
use crate::{init::state::DeploymentEnvironment, util::request_rate_limit::RequestRatePolicy};

fn valid() -> RequestLimitInputs<'static> {
    RequestLimitInputs {
        profile: Some("optimization-v1"),
        deployment: DeploymentEnvironment::Local,
        bind: IpAddr::V4(Ipv4Addr::LOCALHOST),
        disposable: Some("1"),
        origin: Some("https://127.0.0.1:18443"),
        database: Some("postgres://fixture:fixture@127.0.0.1:35432/cyhdev_optimization_test"),
        database_host: None,
        trusted_hops: None,
        trusted_networks: None,
    }
}

#[test]
fn absent_profile_preserves_historical_limits_in_every_deployment() -> anyhow::Result<()> {
    for deployment in [
        DeploymentEnvironment::Local,
        DeploymentEnvironment::Dev,
        DeploymentEnvironment::Staging,
        DeploymentEnvironment::Prod,
    ] {
        let mut inputs = valid();
        inputs.profile = None;
        inputs.deployment = deployment;
        inputs.bind = IpAddr::V4(Ipv4Addr::UNSPECIFIED);
        inputs.origin = Some("https://example.test");
        inputs.database = None;
        inputs.disposable = None;
        assert_eq!(select(inputs)?, RequestRatePolicy::HISTORICAL);
    }
    Ok(())
}

#[test]
fn explicit_disposable_loopback_profile_uses_the_shared_finite_quota() -> anyhow::Result<()> {
    let expected = RequestRatePolicy::bounded(Duration::from_micros(1), 16_384)?;
    assert_eq!(select(valid())?, expected);
    assert_eq!(shared_policy()?, expected);
    let mut ipv6 = valid();
    ipv6.bind = IpAddr::V6(Ipv6Addr::LOCALHOST);
    ipv6.origin = Some("https://[::1]:18443");
    ipv6.database = Some("postgres://fixture:fixture@[::1]:35432/cyhdev_optimization_test");
    ipv6.trusted_hops = Some("0");
    ipv6.trusted_networks = Some("");
    assert_eq!(select(ipv6)?, expected);
    Ok(())
}

#[test]
fn unknown_or_empty_profiles_fail_closed() {
    for profile in [
        "",
        "production",
        "optimization-v2",
        "disabled",
        "optimization-v1 ",
    ] {
        let mut inputs = valid();
        inputs.profile = Some(profile);
        assert!(select(inputs).is_err());
    }
}

#[test]
fn optimization_profile_requires_local_deployment_and_disposable_marker() {
    for deployment in [
        DeploymentEnvironment::Dev,
        DeploymentEnvironment::Staging,
        DeploymentEnvironment::Prod,
    ] {
        let mut inputs = valid();
        inputs.deployment = deployment;
        assert!(select(inputs).is_err());
    }
    for marker in [None, Some(""), Some("true"), Some("0")] {
        let mut inputs = valid();
        inputs.disposable = marker;
        assert!(select(inputs).is_err());
    }
}

#[test]
fn optimization_profile_rejects_wildcard_and_public_listener_addresses() {
    for bind in [
        IpAddr::V4(Ipv4Addr::UNSPECIFIED),
        IpAddr::V6(Ipv6Addr::UNSPECIFIED),
        IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1)),
    ] {
        let mut inputs = valid();
        inputs.bind = bind;
        assert!(select(inputs).is_err());
    }
}

#[test]
fn optimization_profile_rejects_any_forwarded_client_configuration() {
    for (hops, networks) in [
        (Some("1"), None),
        (Some("invalid"), None),
        (Some("0"), Some("127.0.0.1/32")),
        (None, Some("invalid")),
        (Some(" 0 "), None),
    ] {
        let mut inputs = valid();
        inputs.trusted_hops = hops;
        inputs.trusted_networks = networks;
        assert!(select(inputs).is_err());
    }
}

#[test]
fn public_origin_must_be_an_exact_numeric_loopback_origin() {
    for origin in [
        None,
        Some(""),
        Some("https://example.test"),
        Some("https://localhost:18443"),
        Some("https://127.0.0.1.example.test"),
        Some("ftp://127.0.0.1"),
        Some("https://fixture@127.0.0.1"),
        Some("https://127.0.0.1/path"),
        Some("https://127.0.0.1?query=1"),
        Some("https://127.0.0.1#fragment"),
    ] {
        assert!(validate_origin(origin).is_err());
        let mut inputs = valid();
        inputs.origin = origin;
        assert!(select(inputs).is_err());
    }
}

#[test]
fn database_must_remain_in_the_literal_loopback_fixture_namespace() {
    for database in [
        None,
        Some(""),
        Some("postgres://fixture@localhost/cyhdev_optimization_test"),
        Some("postgres://fixture@example.test/cyhdev_optimization_test"),
        Some("postgres://fixture@127.0.0.1/production"),
        Some("postgres://fixture@127.0.0.1/cyhdev_optimization_"),
        Some("postgres://fixture@127.0.0.1/cyhdev_optimization_TEST"),
        Some("postgres://fixture@127.0.0.1/cyhdev_optimization_test?host=/tmp"),
        Some("postgres://fixture@127.0.0.1/cyhdev_optimization_test#override"),
        Some("postgres://fixture@127.0.0.1/cyhdev_optimization_test%2fextra"),
        Some("postgres://fixture@127.0.0.1//cyhdev_optimization_test"),
        Some("mysql://fixture@127.0.0.1/cyhdev_optimization_test"),
    ] {
        assert!(validate_database(database, None).is_err());
        let mut inputs = valid();
        inputs.database = database;
        assert!(select(inputs).is_err());
    }
    let mut inputs = valid();
    inputs.database_host = Some("/tmp");
    assert!(select(inputs).is_err());
}
