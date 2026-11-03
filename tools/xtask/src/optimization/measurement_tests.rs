//! Paired, rotated stage comparisons on synthetic benchmark reports.

use super::{
    measurement::{self, Measurement, Sample},
    rounds,
};

fn measurement(stage: &str, rate: f64, latency: f64) -> Measurement {
    Measurement {
        schema_version: 1,
        stage: stage.into(),
        binary_sha256: if stage == "baseline" { "a" } else { "e" }.repeat(64),
        environment_sha256: "b".repeat(64),
        workload_sha256: "c".repeat(64),
        samples: (0..5)
            .map(|_| Sample {
                requests_per_second: rate,
                p95_ms: latency,
                p99_ms: latency * 2.0,
                failures: 0,
            })
            .collect(),
    }
}

fn accepted(pairs: &[(Measurement, Measurement)]) -> bool {
    match measurement::ratios(pairs) {
        Ok(ratios) => measurement::within(&ratios, 1.05, 0.98),
        Err(_) => false,
    }
}

#[test]
fn faster_candidate_accepts_but_latency_or_throughput_regression_rejects() {
    let round = |rate, latency| {
        (
            measurement("baseline", 100.0, 10.0),
            measurement("bolt", rate, latency),
        )
    };
    assert!(accepted(&[round(110.0, 9.0)]));
    assert!(!accepted(&[round(97.0, 10.0)]));
    assert!(!accepted(&[round(110.0, 11.0)]));
}

/// The first campaign measured each stage once, half an hour apart. Here the
/// host slows by 20% after the first round; per-round ratios still see the
/// candidate's 5% gain, where comparing the baseline's first round with the
/// candidate's last would report a regression.
#[test]
fn per_round_ratios_cancel_host_drift_between_rounds() -> crate::TaskResult<()> {
    let pairs: Vec<_> = [1.0, 0.8, 0.8]
        .into_iter()
        .map(|host| {
            (
                measurement("baseline", 100.0 * host, 10.0 / host),
                measurement("pgo", 105.0 * host, 9.5 / host),
            )
        })
        .collect();
    let ratios = measurement::ratios(&pairs)?;
    assert!((ratios.throughput - 1.05).abs() < 1e-9);
    assert!(measurement::within(&ratios, 1.05, 0.98));
    let unpaired = [(
        measurement("baseline", 100.0, 10.0),
        measurement("pgo", 105.0 * 0.8, 9.5 / 0.8),
    )];
    assert!(!accepted(&unpaired));
    Ok(())
}

#[test]
fn one_outlying_round_does_not_decide_the_comparison() -> crate::TaskResult<()> {
    let pairs = [
        (
            measurement("baseline", 100.0, 10.0),
            measurement("pgo", 110.0, 9.0),
        ),
        (
            measurement("baseline", 100.0, 10.0),
            measurement("pgo", 60.0, 20.0),
        ),
        (
            measurement("baseline", 100.0, 10.0),
            measurement("pgo", 108.0, 9.2),
        ),
    ];
    let ratios = measurement::ratios(&pairs)?;
    assert!((ratios.throughput - 1.08).abs() < 1e-9);
    assert!(measurement::within(&ratios, 1.05, 0.98));
    Ok(())
}

#[test]
fn different_environment_binary_or_invalid_samples_cannot_pass_comparison() {
    let pair = || {
        (
            measurement("baseline", 100.0, 10.0),
            measurement("bolt", 110.0, 9.0),
        )
    };
    let mut changed = pair();
    changed.1.environment_sha256 = "d".repeat(64);
    assert!(!accepted(&[changed]));
    let mut changed = pair();
    changed.1.samples[0].failures = 1;
    assert!(!accepted(&[changed]));
    let mut changed = pair();
    changed.1.samples[0].p95_ms = f64::NAN;
    assert!(!accepted(&[changed]));
    let mut changed = pair();
    changed.1.samples.clear();
    assert!(!accepted(&[changed]));
    let mut swapped = pair();
    swapped.1.binary_sha256 = "f".repeat(64);
    assert!(!accepted(&[pair(), swapped]));
    assert!(!accepted(&[]));
}

#[test]
fn rotation_puts_every_stage_in_every_position_once_per_three_rounds() {
    for position in 0..3 {
        let mut seen: Vec<&str> = (0..3).map(|round| rounds::order(round)[position]).collect();
        seen.sort_unstable();
        assert_eq!(seen, ["baseline", "bolt", "pgo"]);
    }
    assert_eq!(rounds::order(3), rounds::order(0));
}
