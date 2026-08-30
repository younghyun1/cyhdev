//! Stable aggregation and nearest-rank latency percentile calculations.

use std::{collections::BTreeMap, time::Duration};

use crate::{
    executor::types::{ExecutionOutcome, RequestFailure},
    report::Metrics,
};

#[derive(Clone, Copy, Debug)]
pub struct Sample {
    pub index: usize,
    pub latency_ns: u64,
    pub outcome: Result<ExecutionOutcome, RequestFailure>,
}

pub fn aggregate(samples: Vec<Sample>, elapsed: Duration, request_count: u64) -> Metrics {
    let mut latencies = Vec::with_capacity(samples.len());
    let mut successes = 0_u64;
    let mut response_bytes = 0_u64;
    let mut response_checksum = 0xcbf2_9ce4_8422_2325_u64;
    let mut failure_counts = BTreeMap::<String, u64>::new();

    for sample in samples {
        latencies.push(sample.latency_ns);
        let sample_index = u64::try_from(sample.index).unwrap_or(u64::MAX);
        response_checksum ^= sample_index;
        response_checksum = response_checksum.wrapping_mul(0x0000_0100_0000_01b3);
        match sample.outcome {
            Ok(outcome) => {
                successes = successes.saturating_add(1);
                let outcome_bytes = u64::try_from(outcome.response_bytes).unwrap_or(u64::MAX);
                response_bytes = response_bytes.saturating_add(outcome_bytes);
                response_checksum ^= outcome.checksum;
            }
            Err(failure) => {
                let count = failure_counts
                    .entry(failure.as_str().to_owned())
                    .or_default();
                *count = count.saturating_add(1);
            }
        }
    }
    latencies.sort_unstable();
    let failures = request_count.saturating_sub(successes);
    let elapsed_seconds = elapsed.as_secs_f64().max(f64::EPSILON);
    let error_rate_percent = if request_count == 0 {
        100.0
    } else {
        failures as f64 * 100.0 / request_count as f64
    };
    Metrics {
        elapsed_ns: duration_ns(elapsed),
        requests: request_count,
        successes,
        failures,
        error_rate_percent,
        throughput_requests_per_second: request_count as f64 / elapsed_seconds,
        p50_latency_us: percentile_us(&latencies, 50),
        p95_latency_us: percentile_us(&latencies, 95),
        p99_latency_us: percentile_us(&latencies, 99),
        response_bytes,
        response_checksum: format!("fnv1a64:{response_checksum:016x}"),
        failure_counts,
    }
}

fn percentile_us(sorted_latency_ns: &[u64], percentile: usize) -> u64 {
    let rank = percentile
        .saturating_mul(sorted_latency_ns.len())
        .div_ceil(100)
        .saturating_sub(1);
    match sorted_latency_ns.get(rank) {
        Some(latency_ns) => latency_ns.div_ceil(1_000),
        None => 0,
    }
}

pub fn duration_ns(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::percentile_us;

    #[test]
    fn uses_nearest_rank_percentiles_and_rounds_up_to_microseconds() {
        let latencies = [1_001, 2_001, 3_001, 4_001, 5_001];
        assert_eq!(percentile_us(&latencies, 50), 4);
        assert_eq!(percentile_us(&latencies, 95), 6);
        assert_eq!(percentile_us(&latencies, 99), 6);
    }
}
