//! Explicit absolute threshold evaluation for current and saved reports.

use crate::{
    cli::CheckOptions,
    config::{ThresholdConfig, load_report, load_thresholds},
    error::{HarnessError, HarnessResult},
    executor::types::ExecutorKind,
    report::{REPORT_SCHEMA_VERSION, ThresholdEvidence, ThroughputReport, Verdict},
};

pub fn evaluate(report: &ThroughputReport, thresholds: &ThresholdConfig) -> Verdict {
    let mut violations = Vec::new();
    if report.schema_version != REPORT_SCHEMA_VERSION {
        violations.push(format!(
            "report schema is {}, expected {REPORT_SCHEMA_VERSION}",
            report.schema_version
        ));
    }
    if report.metrics.requests != report.configuration.iterations {
        violations.push(format!(
            "report has {} requests but configuration declares {} iterations",
            report.metrics.requests, report.configuration.iterations
        ));
    }
    if report.metrics.successes.saturating_add(report.metrics.failures) != report.metrics.requests {
        violations.push("success and failure counts do not add up to requests".to_owned());
    }
    let categorized_failures = report
        .metrics
        .failure_counts
        .values()
        .copied()
        .fold(0_u64, u64::saturating_add);
    if categorized_failures != report.metrics.failures {
        violations.push("categorized failure counts do not add up to failures".to_owned());
    }
    if report.metrics.elapsed_ns == 0 {
        violations.push("elapsed_ns must be positive".to_owned());
    }
    if !report.metrics.throughput_requests_per_second.is_finite()
        || report.metrics.throughput_requests_per_second <= 0.0
    {
        violations.push("throughput must be finite and positive".to_owned());
    }
    if !report.metrics.error_rate_percent.is_finite()
        || !(0.0..=100.0).contains(&report.metrics.error_rate_percent)
    {
        violations.push("error rate must be finite and in 0..=100".to_owned());
    }
    if report.metrics.p50_latency_us > report.metrics.p95_latency_us
        || report.metrics.p95_latency_us > report.metrics.p99_latency_us
    {
        violations.push("reported latency percentiles are not ordered".to_owned());
    }
    if report.workload.name != thresholds.workload_name {
        violations.push(format!(
            "workload is `{}`, expected `{}`",
            report.workload.name, thresholds.workload_name
        ));
    }
    if report.workload.digest != thresholds.workload_digest {
        violations.push(format!(
            "workload digest is `{}`, expected `{}`",
            report.workload.digest, thresholds.workload_digest
        ));
    }
    if report.environment.declared.label != thresholds.environment_label {
        violations.push(format!(
            "environment is `{}`, expected `{}`",
            report.environment.declared.label, thresholds.environment_label
        ));
    }
    if report.environment.digest != thresholds.environment_digest {
        violations.push(format!(
            "environment digest is `{}`, expected `{}`",
            report.environment.digest, thresholds.environment_digest
        ));
    }
    match &thresholds.observed_environment_digest {
        Some(expected) if report.environment.observed_digest != *expected => {
            violations.push(format!(
                "observed environment digest is `{}`, expected `{expected}`",
                report.environment.observed_digest
            ));
        }
        Some(_) | None => {}
    }
    if report.executor.kind == ExecutorKind::Http {
        match report.environment.declared.configuration.get("target") {
            Some(expected) if *expected == report.executor.target => {}
            Some(expected) => violations.push(format!(
                "executor target is `{}`, but the environment declares `{expected}`",
                report.executor.target
            )),
            None => violations.push(
                "HTTP environment configuration must declare the exact `target`".to_owned(),
            ),
        }
    }
    if report.configuration.compiled_profile != thresholds.compiled_profile {
        violations.push(format!(
            "compiled profile is `{}`, expected `{}`",
            report.configuration.compiled_profile, thresholds.compiled_profile
        ));
    }
    if report.executor.kind.as_str() != thresholds.executor_kind {
        violations.push(format!(
            "executor is `{}`, expected `{}`",
            report.executor.kind.as_str(),
            thresholds.executor_kind
        ));
    }
    if report.metrics.throughput_requests_per_second < thresholds.minimum_requests_per_second {
        violations.push(format!(
            "throughput {:.2} requests/s is below {:.2}",
            report.metrics.throughput_requests_per_second,
            thresholds.minimum_requests_per_second
        ));
    }
    if report.metrics.error_rate_percent > thresholds.maximum_error_rate_percent {
        violations.push(format!(
            "error rate {:.4}% exceeds {:.4}%",
            report.metrics.error_rate_percent, thresholds.maximum_error_rate_percent
        ));
    }
    compare_latency(
        "p50",
        report.metrics.p50_latency_us,
        thresholds.maximum_p50_latency_us,
        &mut violations,
    );
    compare_latency(
        "p95",
        report.metrics.p95_latency_us,
        thresholds.maximum_p95_latency_us,
        &mut violations,
    );
    compare_latency(
        "p99",
        report.metrics.p99_latency_us,
        thresholds.maximum_p99_latency_us,
        &mut violations,
    );
    Verdict {
        passed: violations.is_empty(),
        violations,
    }
}

pub fn check_saved_report(options: &CheckOptions) -> HarnessResult<ThroughputReport> {
    let mut report: ThroughputReport = load_report(&options.report)?;
    let (thresholds, digest) = load_thresholds(&options.thresholds)?;
    report.verdict = evaluate(&report, &thresholds);
    report.thresholds = ThresholdEvidence {
        digest,
        configured: thresholds,
    };
    Ok(report)
}

pub fn enforce(verdict: &Verdict) -> HarnessResult<()> {
    if verdict.passed {
        Ok(())
    } else {
        Err(HarnessError::Regression(verdict.violations.join("; ")))
    }
}

fn compare_latency(name: &str, actual: u64, maximum: u64, violations: &mut Vec<String>) {
    if actual > maximum {
        violations.push(format!(
            "{name} latency {actual} us exceeds {maximum} us"
        ));
    }
}
