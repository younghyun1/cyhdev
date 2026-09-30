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

pub(super) fn compare(
    reference: &Measurement,
    candidate: &Measurement,
    latency_ratio: f64,
    throughput_ratio: f64,
) -> TaskResult<()> {
    validate(reference, &reference.stage, &reference.binary_sha256)?;
    validate(candidate, &candidate.stage, &candidate.binary_sha256)?;
    if reference.environment_sha256 != candidate.environment_sha256
        || reference.workload_sha256 != candidate.workload_sha256
        || reference.samples.len() != candidate.samples.len()
    {
        return Err(TaskError(
            "benchmark environment, workload, or sample count differs".into(),
        ));
    }
    let median = |values: Vec<f64>| {
        let mut values = values;
        values.sort_by(f64::total_cmp);
        values[values.len() / 2]
    };
    let rate = |report: &Measurement| {
        median(
            report
                .samples
                .iter()
                .map(|sample| sample.requests_per_second)
                .collect(),
        )
    };
    let p95 =
        |report: &Measurement| median(report.samples.iter().map(|sample| sample.p95_ms).collect());
    let p99 =
        |report: &Measurement| median(report.samples.iter().map(|sample| sample.p99_ms).collect());
    if rate(candidate) < rate(reference) * throughput_ratio
        || p95(candidate) > p95(reference) * latency_ratio
        || p99(candidate) > p99(reference) * latency_ratio
    {
        return Err(TaskError(format!(
            "{} regresses beyond budgets relative to {}",
            candidate.stage, reference.stage
        )));
    }
    Ok(())
}
