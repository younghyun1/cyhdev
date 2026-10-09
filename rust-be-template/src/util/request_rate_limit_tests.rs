use std::{
    error::Error,
    net::{IpAddr, Ipv4Addr},
    sync::{Arc, atomic::Ordering},
    time::{Duration, Instant},
};

use tokio::sync::Barrier;

use super::{REQUEST_BURST_SIZE, REQUEST_REFILL_INTERVAL, RequestRateLimiter, RequestRatePolicy};

type TestResult = Result<(), Box<dyn Error>>;

fn client(index: u32) -> IpAddr {
    IpAddr::V4(Ipv4Addr::from(index))
}

async fn exhaust(limiter: &RequestRateLimiter, ip: IpAddr, now: Instant) {
    for _ in 0..REQUEST_BURST_SIZE {
        assert!(limiter.check_at(ip, now).await.is_ok());
    }
}

#[tokio::test]
async fn historical_burst_and_exact_refill_boundary_are_preserved() {
    let limiter = RequestRateLimiter::new();
    let now = Instant::now();
    let ip = client(1);
    exhaust(&limiter, ip, now).await;
    let rejection = limiter.check_at(ip, now).await;
    assert!(matches!(rejection, Err(rejection) if !rejection.saturated
        && rejection.retry_after == REQUEST_REFILL_INTERVAL));
    let early = now + REQUEST_REFILL_INTERVAL - Duration::from_nanos(1);
    let rejection = limiter.check_at(ip, early).await;
    assert!(
        matches!(rejection, Err(rejection) if rejection.retry_after == Duration::from_nanos(1))
    );
    assert!(
        limiter
            .check_at(ip, now + REQUEST_REFILL_INTERVAL)
            .await
            .is_ok()
    );
    assert!(
        limiter
            .check_at(ip, now + REQUEST_REFILL_INTERVAL)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn relentless_manual_clicking_at_ten_per_second_stays_admitted() {
    let limiter = RequestRateLimiter::new();
    let now = Instant::now();
    for tick in 0..36_000 {
        let click = now + Duration::from_millis(tick * 100);
        assert!(limiter.check_at(client(1), click).await.is_ok());
    }
    assert_eq!(limiter.clients.len(), 1);
}

#[tokio::test]
async fn independent_clients_and_ipv6_networks_have_independent_budgets() -> TestResult {
    let limiter = RequestRateLimiter::new();
    let now = Instant::now();
    let first: IpAddr = "2001:db8:1:2::1".parse()?;
    let same_network: IpAddr = "2001:db8:1:2:ffff::9".parse()?;
    let other_network: IpAddr = "2001:db8:1:3::1".parse()?;
    exhaust(&limiter, first, now).await;
    assert!(limiter.check_at(same_network, now).await.is_err());
    assert!(limiter.check_at(other_network, now).await.is_ok());
    assert!(limiter.check_at(client(1), now).await.is_ok());
    assert!(limiter.check_at(client(2), now).await.is_ok());
    Ok(())
}

#[tokio::test]
async fn mapped_ipv4_uses_the_native_ipv4_budget() -> TestResult {
    let limiter = RequestRateLimiter::new();
    let now = Instant::now();
    let ip: IpAddr = "192.0.2.1".parse()?;
    exhaust(&limiter, ip, now).await;
    assert!(
        limiter
            .check_at("::ffff:192.0.2.1".parse()?, now)
            .await
            .is_err()
    );
    assert!(
        limiter
            .check_at("::ffff:192.0.2.2".parse()?, now)
            .await
            .is_ok()
    );
    Ok(())
}

#[tokio::test]
async fn capacity_pressure_never_evicts_or_resets_existing_client_debt() {
    let limiter = RequestRateLimiter::with_capacity(2, RequestRatePolicy::HISTORICAL);
    let now = Instant::now();
    exhaust(&limiter, client(1), now).await;
    exhaust(&limiter, client(2), now).await;
    for index in 3..1_000 {
        let rejected = limiter.check_at(client(index), now).await;
        assert!(matches!(rejected, Err(rejection) if rejection.saturated
            && rejection.retry_after == Duration::from_secs(1)));
    }
    assert_eq!(limiter.clients.len(), 2);
    assert_eq!(limiter.active_slots.load(Ordering::Acquire), 2);
    let rejected = limiter.check_at(client(1), now).await;
    assert!(matches!(rejected, Err(rejection) if !rejection.saturated));
    assert!(
        limiter
            .check_at(client(1), now + REQUEST_REFILL_INTERVAL)
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn capacity_recovers_only_when_request_debt_is_fully_replenished() {
    let limiter = RequestRateLimiter::with_capacity(1, RequestRatePolicy::HISTORICAL);
    let now = Instant::now();
    exhaust(&limiter, client(1), now).await;
    let recovery = now + REQUEST_REFILL_INTERVAL * REQUEST_BURST_SIZE;
    let early = recovery - Duration::from_nanos(1);
    assert!(limiter.check_at(client(2), early).await.is_err());
    // A saturated spray cannot trigger a second table sweep inside one second.
    assert!(limiter.check_at(client(2), recovery).await.is_err());
    assert!(
        limiter
            .check_at(client(2), early + Duration::from_secs(1))
            .await
            .is_ok()
    );
    assert_eq!(limiter.clients.len(), 1);
    assert_eq!(limiter.active_slots.load(Ordering::Acquire), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_distinct_client_admission_respects_the_reserved_capacity() -> TestResult {
    let limiter = Arc::new(RequestRateLimiter::with_capacity(
        8,
        RequestRatePolicy::HISTORICAL,
    ));
    let barrier = Arc::new(Barrier::new(64));
    let now = Instant::now();
    let mut jobs = Vec::new();
    for index in 0..64 {
        let limiter = Arc::clone(&limiter);
        let barrier = Arc::clone(&barrier);
        jobs.push(tokio::spawn(async move {
            barrier.wait().await;
            limiter.check_at(client(index), now).await
        }));
    }
    let mut admitted = 0;
    for job in jobs {
        if job.await?.is_ok() {
            admitted += 1;
        }
    }
    assert_eq!(admitted, 8);
    assert_eq!(limiter.clients.len(), 8);
    assert_eq!(limiter.active_slots.load(Ordering::Acquire), 8);
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_requests_for_one_client_share_one_reserved_slot() -> TestResult {
    let limiter = Arc::new(RequestRateLimiter::with_capacity(
        1,
        RequestRatePolicy::HISTORICAL,
    ));
    let barrier = Arc::new(Barrier::new(2_048));
    let now = Instant::now();
    let mut jobs = Vec::new();
    for _ in 0..2_048 {
        let limiter = Arc::clone(&limiter);
        let barrier = Arc::clone(&barrier);
        jobs.push(tokio::spawn(async move {
            barrier.wait().await;
            limiter.check_at(client(1), now).await
        }));
    }
    let mut admitted = 0;
    for job in jobs {
        match job.await? {
            Ok(()) => admitted += 1,
            Err(rejection) => assert!(!rejection.saturated),
        }
    }
    assert_eq!(admitted, REQUEST_BURST_SIZE);
    assert_eq!(limiter.clients.len(), 1);
    assert_eq!(limiter.active_slots.load(Ordering::Acquire), 1);
    Ok(())
}

#[test]
fn abandoned_reservations_return_the_capacity_slot() {
    let limiter = RequestRateLimiter::with_capacity(1, RequestRatePolicy::HISTORICAL);
    let reservation = limiter.try_reserve();
    assert!(reservation.is_some());
    assert!(limiter.try_reserve().is_none());
    drop(reservation);
    assert_eq!(limiter.active_slots.load(Ordering::Acquire), 0);
    assert!(limiter.try_reserve().is_some());
}
