//! Same-environment, repeated artifact comparisons before publishing a candidate.

use serde::{Deserialize, Serialize};

use crate::{TaskError, TaskResult};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Measurement {
    pub schema_version: u8,
    pub stage: String,
    pub binary_sha256: String,
    pub environment_sha256: String,
    pub workload_sha256: String,
    pub samples: Vec<Sample>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Sample {
    pub requests_per_second: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub failures: u64,
}

pub(super) fn validate(report: &Measurement, stage: &str, digest: &str) -> TaskResult<()> {
    if report.schema_version != 1
        || report.stage != stage
        || report.binary_sha256 != digest
        || !sha256(&report.environment_sha256)
        || !sha256(&report.workload_sha256)
        || !(5..=100).contains(&report.samples.len())
        || report.samples.iter().any(|sample| {
            sample.failures != 0
                || !positive(sample.requests_per_second)
                || !positive(sample.p95_ms)
                || !positive(sample.p99_ms)
                || sample.p99_ms < sample.p95_ms
        })
    {
        return Err(TaskError(
            "benchmark requires matching identity and 5-100 valid, error-free samples".into(),
        ));
    }
    Ok(())
}

fn sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn positive(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

/// Medians of one benchmark run.
#[derive(Clone, Copy, Debug, Serialize)]
pub(super) struct Summary {
    pub requests_per_second: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
}

/// Candidate-to-reference ratios; throughput above 1 and latency below 1
/// favor the candidate.
#[derive(Clone, Copy, Debug, Serialize)]
pub(super) struct Ratios {
    pub throughput: f64,
    pub p95: f64,
    pub p99: f64,
}

/// Median of a list, averaging the middle pair of an even-length list.
fn median(mut values: Vec<f64>) -> Option<f64> {
    values.sort_by(f64::total_cmp);
    let middle = values.len() / 2;
    match values.len() {
        0 => None,
        length if length % 2 == 0 => Some((values[middle - 1] + values[middle]) / 2.0),
        _ => Some(values[middle]),
    }
}

pub(super) fn summary(report: &Measurement) -> TaskResult<Summary> {
    let metric = |value: fn(&Sample) -> f64| median(report.samples.iter().map(value).collect());
    match (
        metric(|sample| sample.requests_per_second),
        metric(|sample| sample.p95_ms),
        metric(|sample| sample.p99_ms),
    ) {
        (Some(requests_per_second), Some(p95_ms), Some(p99_ms)) => Ok(Summary {
            requests_per_second,
            p95_ms,
            p99_ms,
        }),
        _ => Err(TaskError("benchmark report has no samples".into())),
    }
}

/// Compares two stages measured in the same rounds, as `(reference, candidate)` pairs.
///
/// Each round contributes one ratio per metric from runs taken minutes apart,
/// so host drift between rounds (clock and idle-state management, thermal
/// state, other load) affects both sides of that ratio alike; the median
/// across rounds then discards one outlying round. Comparing single runs taken
/// half an hour apart instead mistook such drift for a binary regression.
pub(super) fn ratios(rounds: &[(Measurement, Measurement)]) -> TaskResult<Ratios> {
    let Some((first_reference, first_candidate)) = rounds.first() else {
        return Err(TaskError("comparison requires at least one round".into()));
    };
    let (mut throughput, mut p95, mut p99) = (Vec::new(), Vec::new(), Vec::new());
    for (reference, candidate) in rounds {
        validate(
            reference,
            &first_reference.stage,
            &first_reference.binary_sha256,
        )?;
        validate(
            candidate,
            &first_candidate.stage,
            &first_candidate.binary_sha256,
        )?;
        if [reference, candidate].iter().any(|report| {
            report.environment_sha256 != first_reference.environment_sha256
                || report.workload_sha256 != first_reference.workload_sha256
                || report.samples.len() != first_reference.samples.len()
        }) {
            return Err(TaskError(
                "benchmark environment, workload, or sample count differs".into(),
            ));
        }
        let (reference, candidate) = (summary(reference)?, summary(candidate)?);
        throughput.push(candidate.requests_per_second / reference.requests_per_second);
        p95.push(candidate.p95_ms / reference.p95_ms);
        p99.push(candidate.p99_ms / reference.p99_ms);
    }
    match (median(throughput), median(p95), median(p99)) {
        (Some(throughput), Some(p95), Some(p99)) => Ok(Ratios {
            throughput,
            p95,
            p99,
        }),
        _ => Err(TaskError("comparison requires at least one round".into())),
    }
}

/// Whether a candidate keeps the configured throughput and latency budgets.
pub(super) fn within(ratios: &Ratios, latency_ratio: f64, throughput_ratio: f64) -> bool {
    ratios.throughput >= throughput_ratio
        && ratios.p95 <= latency_ratio
        && ratios.p99 <= latency_ratio
}
