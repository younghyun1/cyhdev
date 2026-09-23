//! Unit coverage for the bounded view-delta buffer.

use super::{ViewAdmission, ViewDeltaBuffer, ViewRecordOutcome};
use std::sync::atomic::{AtomicUsize, Ordering};
use uuid::Uuid;

#[tokio::test]
async fn full_buffer_still_accepts_an_existing_key() {
    let views = ViewDeltaBuffer::new("test", 4);
    let existing = Uuid::from_u128(1);
    for value in 1..=4_u128 {
        assert_eq!(
            views.try_record(Uuid::from_u128(value)).await,
            ViewAdmission::Recorded
        );
    }
    assert_eq!(views.try_record(existing).await, ViewAdmission::Recorded);
    assert_eq!(views.pending(existing).await, 2);
    assert_eq!(views.try_record(Uuid::now_v7()).await, ViewAdmission::Full);
}

#[tokio::test]
async fn flush_settles_applied_deltas_and_keeps_failed_ones() {
    let views = ViewDeltaBuffer::new("test", 8);
    let applied = Uuid::from_u128(1);
    let failed = Uuid::from_u128(2);
    for _ in 0..3 {
        views.try_record(applied).await;
    }
    views.try_record(failed).await;
    let flushed = views
        .flush(|pending: Vec<(Uuid, i64)>| async move {
            Ok::<_, ()>(
                pending
                    .into_iter()
                    .filter(|(id, _)| *id == applied)
                    .collect(),
            )
        })
        .await;
    assert_eq!(flushed, Ok(3));
    assert_eq!(views.pending(applied).await, 0);
    assert_eq!(views.pending(failed).await, 1);
    assert!(views.epoch().is_multiple_of(2));
    assert_eq!(views.epoch(), 2);
}

#[tokio::test]
async fn full_buffer_flushes_before_falling_back_to_a_direct_write() {
    let views = ViewDeltaBuffer::new("test", 1);
    views.try_record(Uuid::from_u128(1)).await;
    let increments = AtomicUsize::new(0);
    // A sink that never applies anything forces the direct-write fallback.
    let outcome = views
        .record(
            Uuid::from_u128(2),
            |_| async { Ok::<_, ()>(Vec::new()) },
            |_| async {
                increments.fetch_add(1, Ordering::Relaxed);
                Ok(())
            },
        )
        .await;
    assert!(matches!(outcome, Ok(ViewRecordOutcome::Persisted)));
    assert_eq!(increments.load(Ordering::Relaxed), 1);

    // A sink that applies the batch frees the slot, so the view is buffered.
    let outcome = views
        .record(
            Uuid::from_u128(3),
            |pending| async move { Ok::<_, ()>(pending) },
            |_| async { Ok(()) },
        )
        .await;
    assert!(matches!(outcome, Ok(ViewRecordOutcome::Buffered)));
    assert_eq!(views.pending(Uuid::from_u128(3)).await, 1);
}
