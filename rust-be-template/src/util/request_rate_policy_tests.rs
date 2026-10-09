use std::{
    net::{IpAddr, Ipv4Addr},
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

use super::{MAX_REQUEST_RATE_CLIENTS, RequestRateLimiter, RequestRatePolicy};

fn client(index: u32) -> IpAddr {
    IpAddr::V4(Ipv4Addr::from(index))
}

#[tokio::test]
async fn optimization_policy_still_exhausts_and_refills_a_finite_quota() -> anyhow::Result<()> {
    let policy = RequestRatePolicy::bounded(Duration::from_micros(1), 16_384)?;
    let limiter = RequestRateLimiter::with_policy(policy);
    let now = Instant::now();
    for _ in 0..16_384 {
        assert!(limiter.check_at(client(1), now).await.is_ok());
    }
    let rejected = limiter.check_at(client(1), now).await;
    assert!(matches!(rejected, Err(rejection) if !rejection.saturated
        && rejection.retry_after == Duration::from_micros(1)));
    let refill = now + Duration::from_micros(1);
    assert!(limiter.check_at(client(1), refill).await.is_ok());
    assert!(limiter.check_at(client(1), refill).await.is_err());
    Ok(())
}

#[tokio::test]
async fn optimization_policy_preserves_the_table_and_entry_memory_bounds() -> anyhow::Result<()> {
    let policy = RequestRatePolicy::bounded(Duration::from_micros(1), 16_384)?;
    let limiter = RequestRateLimiter::with_policy(policy);
    let now = Instant::now();
    for index in 0..MAX_REQUEST_RATE_CLIENTS as u32 {
        assert!(limiter.check_at(client(index), now).await.is_ok());
    }
    let rejected = limiter
        .check_at(client(MAX_REQUEST_RATE_CLIENTS as u32), now)
        .await;
    assert!(matches!(rejected, Err(rejection) if rejection.saturated));
    assert_eq!(limiter.max_clients, MAX_REQUEST_RATE_CLIENTS);
    assert_eq!(limiter.clients.len(), MAX_REQUEST_RATE_CLIENTS);
    assert_eq!(
        limiter.active_slots.load(Ordering::Acquire),
        MAX_REQUEST_RATE_CLIENTS
    );
    assert!(std::mem::size_of::<(IpAddr, Instant)>() <= 64);
    Ok(())
}

#[test]
fn alternate_policies_cannot_remove_the_finite_request_bound() {
    for (interval, burst) in [
        (Duration::ZERO, 1),
        (Duration::from_nanos(999), 1),
        (Duration::from_secs(1), 1),
        (Duration::from_micros(1), 0),
        (Duration::from_micros(1), 16_385),
    ] {
        assert!(RequestRatePolicy::bounded(interval, burst).is_err());
    }
}

#[tokio::test]
async fn cold_browser_contexts_share_one_budget_across_successive_page_visits() -> anyhow::Result<()>
{
    let optimization = RequestRatePolicy::bounded(Duration::from_micros(1), 16_384)?;
    for (policy, expected_admitted, expected_rejected_page) in [
        (RequestRatePolicy::HISTORICAL, 1_024, Some("/verify-email")),
        (optimization, 1_152, None),
    ] {
        let limiter = RequestRateLimiter::with_policy(policy);
        let now = Instant::now();
        let mut admitted = 0;
        let mut first_rejected_page = None;
        for page in [
            "/",
            "/about",
            "/about-blog",
            "/find-password",
            "/reset-password",
            "/verify-email",
        ] {
            for _context in 0..8 {
                // Synthetic document, asset, and API fanout isolates cold-context
                // accounting; 24 is not a claim about the current asset inventory.
                for _request in 0..24 {
                    match limiter.check_at(client(1), now).await {
                        Ok(()) => admitted += 1,
                        Err(_) => {
                            first_rejected_page.get_or_insert(page);
                        }
                    }
                }
            }
        }
        assert_eq!(admitted, expected_admitted);
        assert_eq!(first_rejected_page, expected_rejected_page);
    }
    Ok(())
}
