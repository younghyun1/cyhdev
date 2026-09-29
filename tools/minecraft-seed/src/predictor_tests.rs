//! Persistent sampling parity, admission bounds, cancellation and development timings.

use super::*;
use std::{future::Future, task::Poll, time::Instant};

fn request(large_biomes: bool, x: i32, z: i32, step: u32) -> PredictionRequest {
    PredictionRequest {
        seed: 1,
        large_biomes,
        y: 64,
        min_x: x,
        min_z: z,
        width: 64,
        height: 64,
        step,
    }
}

#[tokio::test]
async fn persistent_sampler_matches_fresh_sampling_across_tiles_and_presets() -> Result<(), Error> {
    let predictor = Predictor::new()?;
    for (large, x, z, step, y) in [
        (false, -256, -512, 4, 64),
        (false, 1024, -512, 8, 64),
        (false, -256, -512, 4, 64),
        (false, -253, -509, 3, -63),
        (false, -256, -512, 4, 319),
        (true, 768, -1536, 4, 64),
        (false, 0, 0, 64, 64),
    ] {
        let mut input = request(large, x, z, step);
        input.y = y;
        let expected = crate::predict(&input)?;
        let actual = predictor.predict(input).await?;
        assert_eq!(actual.cells.len(), 4096);
        for (actual, expected) in actual.cells.iter().zip(expected.cells) {
            assert_eq!(
                (actual.x, actual.z, &actual.biome),
                (expected.x, expected.z, &expected.biome)
            );
        }
    }
    Ok(())
}

#[tokio::test]
async fn admission_is_bounded_and_disconnection_is_explicit() {
    let (sender, receiver) = mpsc::sync_channel(QUEUE_CAPACITY);
    for _ in 0..QUEUE_CAPACITY {
        let (reply, _response) = oneshot::channel();
        assert!(
            sender
                .try_send(Job {
                    request: request(false, 0, 0, 4),
                    reply
                })
                .is_ok()
        );
    }
    let predictor = Predictor { sender };
    assert!(matches!(
        predictor.predict(request(false, 0, 0, 4)).await,
        Err(Error::Busy)
    ));
    drop(receiver);
    assert!(matches!(
        predictor.predict(request(false, 0, 0, 4)).await,
        Err(Error::Unavailable)
    ));
}

#[tokio::test]
async fn dropping_a_waiter_marks_queued_work_cancelled() {
    let (sender, receiver) = mpsc::sync_channel(1);
    let predictor = Predictor { sender };
    let mut pending = Box::pin(predictor.predict(request(false, 0, 0, 4)));
    std::future::poll_fn(|cx| {
        assert!(pending.as_mut().poll(cx).is_pending());
        Poll::Ready(())
    })
    .await;
    drop(pending);
    drop(predictor);
    assert!(next(&receiver).is_none());
}

#[test]
fn active_sampling_observes_cancellation_before_density_work() {
    let router = generator::router(1, false);
    let mut sampler = MultiNoiseSampler::generate(&router);
    assert!(matches!(
        generator::sample(&request(false, 0, 0, 4), &mut sampler, || true),
        Err(Error::Cancelled)
    ));
}

#[tokio::test]
#[ignore = "development timing, run explicitly without a live server"]
async fn measure_persistent_tiles() -> Result<(), Error> {
    let predictor = Predictor::new()?;
    for large in [false, true] {
        let mut elapsed = Vec::new();
        for i in 0..17 {
            let start = Instant::now();
            let result = predictor.predict(request(large, i * 256, -1024, 4)).await?;
            assert_eq!(result.cells.len(), 4096);
            elapsed.push(start.elapsed());
        }
        let cold = elapsed.remove(0);
        elapsed.sort();
        println!(
            "large_biomes={large} cold={cold:?} warm_median={:?} warm_max={:?} samples=4096",
            elapsed[elapsed.len() / 2],
            elapsed.last()
        );
    }
    Ok(())
}
