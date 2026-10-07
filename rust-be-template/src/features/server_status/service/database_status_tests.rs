use std::{
    future::pending,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use tokio::sync::{Semaphore, oneshot};

use super::{DatabaseStatus, DatabaseStatusProbe, major_version};

const STATUS: DatabaseStatus = DatabaseStatus {
    major_version: 18,
    latency: Duration::from_millis(1),
};

fn single_probe(deadline: Duration) -> Arc<DatabaseStatusProbe> {
    Arc::new(DatabaseStatusProbe {
        slots: Arc::new(Semaphore::new(1)),
        deadline,
    })
}

async fn wait_for_probe_release(probe: &DatabaseStatusProbe) -> anyhow::Result<()> {
    tokio::time::timeout(Duration::from_secs(1), async {
        while probe.slots.available_permits() == 0 {
            tokio::task::yield_now().await;
        }
    })
    .await?;
    Ok(())
}

#[test]
fn version_numbers_reduce_to_the_major_version() {
    assert_eq!(major_version(180_001), Some(18));
    assert_eq!(major_version(100_000), Some(10));
    assert_eq!(major_version(90_624), None);
    assert_eq!(major_version(-1), None);
}

#[tokio::test]
async fn every_request_measures_a_new_query_and_does_not_retain_failures() -> anyhow::Result<()> {
    let probe = DatabaseStatusProbe::new();
    let calls = Arc::new(AtomicUsize::new(0));
    for expected in 1..=3 {
        let calls = Arc::clone(&calls);
        let status = probe
            .measure(move || async move {
                let call = calls.fetch_add(1, Ordering::SeqCst) + 1;
                Ok(DatabaseStatus {
                    latency: Duration::from_millis(call as u64),
                    ..STATUS
                })
            })
            .await?;
        assert_eq!(status.latency, Duration::from_millis(expected));
    }
    for _ in 0..2 {
        let calls = Arc::clone(&calls);
        let status = probe
            .measure(move || async move {
                calls.fetch_add(1, Ordering::SeqCst);
                Err(anyhow::anyhow!("pool timed out"))
            })
            .await;
        assert!(status.is_err());
    }
    assert_eq!(calls.load(Ordering::SeqCst), 5);
    assert_eq!(probe.measure(|| async { Ok(STATUS) }).await?, STATUS);
    Ok(())
}

#[tokio::test]
async fn saturation_refuses_work_without_running_or_queueing_it() -> anyhow::Result<()> {
    let probe = single_probe(Duration::from_secs(1));
    let (started_tx, started_rx) = oneshot::channel();
    let (release_tx, release_rx) = oneshot::channel();
    let running = Arc::clone(&probe);
    let first = tokio::spawn(async move {
        running
            .measure(move || async move {
                let _ = started_tx.send(());
                release_rx.await?;
                Ok(STATUS)
            })
            .await
    });
    started_rx.await?;
    let rejected_calls = Arc::new(AtomicUsize::new(0));
    let calls = Arc::clone(&rejected_calls);
    let rejected = probe
        .measure(move || async move {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(STATUS)
        })
        .await;
    assert!(rejected.is_err());
    assert_eq!(rejected_calls.load(Ordering::SeqCst), 0);
    assert!(release_tx.send(()).is_ok());
    assert_eq!(first.await??, STATUS);
    assert_eq!(probe.measure(|| async { Ok(STATUS) }).await?, STATUS);
    Ok(())
}

#[tokio::test]
async fn timeout_keeps_the_slot_until_the_database_operation_finishes() -> anyhow::Result<()> {
    let probe = single_probe(Duration::from_millis(10));
    let (release_tx, release_rx) = oneshot::channel();
    let outcome = probe
        .measure(move || async move {
            release_rx.await?;
            Ok(STATUS)
        })
        .await;
    assert!(outcome.is_err());
    assert_eq!(probe.slots.available_permits(), 0);
    assert!(probe.measure(|| async { Ok(STATUS) }).await.is_err());
    assert!(release_tx.send(()).is_ok());
    wait_for_probe_release(&probe).await?;
    assert_eq!(probe.measure(|| async { Ok(STATUS) }).await?, STATUS);
    Ok(())
}

#[tokio::test]
async fn caller_cancellation_keeps_the_slot_until_the_probe_finishes() -> anyhow::Result<()> {
    let probe = single_probe(Duration::from_secs(1));
    let (started_tx, started_rx) = oneshot::channel();
    let (release_tx, release_rx) = oneshot::channel();
    let running = Arc::clone(&probe);
    let caller = tokio::spawn(async move {
        running
            .measure(move || async move {
                let _ = started_tx.send(());
                release_rx.await?;
                Ok(STATUS)
            })
            .await
    });
    started_rx.await?;
    caller.abort();
    assert!(caller.await.is_err());
    assert_eq!(probe.slots.available_permits(), 0);
    assert!(release_tx.send(()).is_ok());
    wait_for_probe_release(&probe).await?;
    assert_eq!(probe.measure(|| async { Ok(STATUS) }).await?, STATUS);
    Ok(())
}

#[tokio::test]
async fn saturation_returns_before_the_probe_deadline() -> anyhow::Result<()> {
    let probe = single_probe(Duration::from_secs(1));
    let permit = Arc::clone(&probe.slots).try_acquire_owned()?;
    let result = tokio::time::timeout(
        Duration::from_millis(10),
        probe.measure(pending::<anyhow::Result<DatabaseStatus>>),
    )
    .await?;
    assert!(result.is_err());
    drop(permit);
    Ok(())
}
