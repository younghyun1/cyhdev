//! Bounded concurrent replay and percentile aggregation.

use std::{
    hint::black_box,
    sync::{Arc, Barrier},
    thread,
    time::{Duration, Instant},
};

use tracing::info;

use crate::{
    check,
    cli::RunOptions,
    config::{
        EnvironmentConfig, RequestSpec, WorkloadConfig, load_environment, load_thresholds,
        load_workload,
    },
    error::{HarnessError, HarnessResult},
    executor::engine::Executor,
    hardware,
    report::{
        EnvironmentMetadata, ExecutorMetadata, REPORT_SCHEMA_VERSION, RunConfiguration,
        ThresholdEvidence, ThroughputReport, WorkloadMetadata, compiled_profile,
    },
    statistics::{Sample, aggregate, duration_ns},
};

struct CompletionSignal {
    barrier: Arc<Barrier>,
}

impl Drop for CompletionSignal {
    fn drop(&mut self) {
        let _result = self.barrier.wait();
    }
}

pub fn run(options: &RunOptions) -> HarnessResult<ThroughputReport> {
    let (workload, workload_digest) = load_workload(&options.workload)?;
    let (environment, environment_digest) = load_environment(&options.environment)?;
    let threshold_input = options
        .thresholds
        .as_deref()
        .map(load_thresholds)
        .transpose()?;
    let executor = Executor::new(options.target.as_deref(), &workload)?;
    validate_executor_declaration(&options.environment, &environment, &executor)?;
    let schedule = build_schedule(&workload)?;

    info!(
        workload = %workload.name,
        executor = executor.kind().as_str(),
        concurrency = workload.concurrency,
        iterations = workload.iterations,
        "Starting bounded throughput replay"
    );
    warm_up(&executor, &workload, &schedule);
    let (samples, elapsed) = replay(executor.clone(), &workload, schedule)?;
    let metrics = aggregate(samples, elapsed, workload.iterations);

    let hardware = hardware::observe();
    let executor_kind = executor.kind();
    let executor_target = executor.label();
    let resolved_address = executor.resolved_address();
    let implementation_digest = crate::implementation::digest();
    let observed_environment_digest = hardware::environment_digest(
        &environment_digest,
        &hardware,
        compiled_profile(),
        implementation_digest,
        executor_kind.as_str(),
        &executor_target,
        resolved_address.as_deref(),
    )?;
    let mut report = ThroughputReport {
        schema_version: REPORT_SCHEMA_VERSION,
        workload: WorkloadMetadata {
            name: workload.name.clone(),
            digest: workload_digest,
        },
        executor: ExecutorMetadata {
            kind: executor_kind,
            target: executor_target,
            resolved_address,
        },
        environment: EnvironmentMetadata {
            digest: environment_digest,
            observed_digest: observed_environment_digest,
            declared: environment,
        },
        hardware,
        configuration: RunConfiguration {
            harness_version: env!("CARGO_PKG_VERSION").to_owned(),
            compiled_profile: compiled_profile().to_owned(),
            implementation_digest: implementation_digest.to_owned(),
            iterations: workload.iterations,
            warmup_iterations: workload.warmup_iterations,
            concurrency: workload.concurrency,
            timeout_ms: workload.timeout_ms,
            max_response_bytes: workload.max_response_bytes,
        },
        metrics,
        thresholds: threshold_input
            .as_ref()
            .map(|(configured, digest)| ThresholdEvidence {
                digest: digest.clone(),
                configured: configured.clone(),
            }),
        verdict: None,
    };
    if let Some((thresholds, _)) = &threshold_input {
        report.verdict = Some(check::evaluate(&report, thresholds));
    }
    Ok(report)
}

fn validate_executor_declaration(
    environment_path: &std::path::Path,
    environment: &EnvironmentConfig,
    executor: &Executor,
) -> HarnessResult<()> {
    if executor.kind() != crate::executor::types::ExecutorKind::Http {
        return Ok(());
    }
    crate::environment_evidence::validate(environment_path, environment)?;
    let actual = executor.label();
    match environment.configuration.get("target") {
        Some(expected) if expected == &actual => Ok(()),
        Some(expected) => Err(HarnessError::Configuration {
            path: environment_path.to_path_buf(),
            detail: format!(
                "declared target `{expected}` does not match executor target `{actual}`"
            ),
        }),
        None => Err(HarnessError::Configuration {
            path: environment_path.to_path_buf(),
            detail: "HTTP environments must declare the exact `target`".to_owned(),
        }),
    }
}

fn build_schedule(workload: &WorkloadConfig) -> HarnessResult<Vec<usize>> {
    let capacity = workload
        .requests
        .iter()
        .try_fold(0_usize, |total, request| {
            let weight = usize::try_from(request.weight).map_err(|_source| {
                HarnessError::Arguments("request weight does not fit this platform".to_owned())
            })?;
            total.checked_add(weight).ok_or_else(|| {
                HarnessError::Arguments("combined request weight overflowed usize".to_owned())
            })
        })?;
    let mut schedule = Vec::with_capacity(capacity);
    for (request_index, request) in workload.requests.iter().enumerate() {
        for _occurrence in 0..request.weight {
            schedule.push(request_index);
        }
    }
    Ok(schedule)
}

fn warm_up(executor: &Executor, workload: &WorkloadConfig, schedule: &[usize]) {
    for index in 0..workload.warmup_iterations {
        let request = request_for(index, workload, schedule);
        match request {
            Some(request) => drop(black_box(executor.execute(request))),
            None => break,
        }
    }
}

fn replay(
    executor: Executor,
    workload: &WorkloadConfig,
    schedule: Vec<usize>,
) -> HarnessResult<(Vec<Sample>, Duration)> {
    let iterations = usize::try_from(workload.iterations).map_err(|_source| {
        HarnessError::Arguments("iterations do not fit this platform".to_owned())
    })?;
    let executor = Arc::new(executor);
    let requests = Arc::new(workload.requests.clone());
    let schedule = Arc::new(schedule);
    let ready_barrier = Arc::new(Barrier::new(workload.concurrency.saturating_add(1)));
    let start_barrier = Arc::new(Barrier::new(workload.concurrency.saturating_add(1)));
    let completion_barrier = Arc::new(Barrier::new(workload.concurrency.saturating_add(1)));
    let mut workers = Vec::with_capacity(workload.concurrency);

    for worker in 0..workload.concurrency {
        let executor = Arc::clone(&executor);
        let requests = Arc::clone(&requests);
        let schedule = Arc::clone(&schedule);
        let ready_barrier = Arc::clone(&ready_barrier);
        let start_barrier = Arc::clone(&start_barrier);
        let completion_barrier = Arc::clone(&completion_barrier);
        let concurrency = workload.concurrency;
        let handle = thread::Builder::new()
            .name(format!("throughput-{worker}"))
            .spawn(move || {
                ready_barrier.wait();
                start_barrier.wait();
                let _completion = CompletionSignal {
                    barrier: completion_barrier,
                };
                measure_worker(
                    worker,
                    concurrency,
                    iterations,
                    executor.as_ref(),
                    &requests,
                    &schedule,
                )
            })
            .map_err(|source| HarnessError::ThreadSpawn { worker, source })?;
        workers.push((worker, handle));
    }

    // Do not charge thread creation or joins to the service. Prepared workers
    // start together; the completion barrier stops time at the last request.
    ready_barrier.wait();
    let started = Instant::now();
    start_barrier.wait();
    completion_barrier.wait();
    let elapsed = started.elapsed();
    let mut samples = Vec::with_capacity(iterations);
    for (worker, handle) in workers {
        let worker_result = handle
            .join()
            .map_err(|_payload| HarnessError::WorkerTerminated { worker })?;
        let mut worker_samples = worker_result?;
        samples.append(&mut worker_samples);
    }
    samples.sort_unstable_by_key(|sample| sample.index);
    Ok((samples, elapsed))
}

fn measure_worker(
    worker: usize,
    concurrency: usize,
    iterations: usize,
    executor: &Executor,
    requests: &[RequestSpec],
    schedule: &[usize],
) -> HarnessResult<Vec<Sample>> {
    if concurrency == 0 || schedule.is_empty() {
        return Err(HarnessError::WorkerInvariant {
            worker,
            detail: "concurrency and the weighted schedule must be nonzero".to_owned(),
        });
    }
    let capacity = iterations.saturating_sub(worker).div_ceil(concurrency);
    let mut samples = Vec::with_capacity(capacity);
    for index in (worker..iterations).step_by(concurrency) {
        let request_index = match schedule.get(index % schedule.len()) {
            Some(request_index) => *request_index,
            None => {
                return Err(HarnessError::WorkerInvariant {
                    worker,
                    detail: "the request schedule index was out of bounds".to_owned(),
                });
            }
        };
        let request = match requests.get(request_index) {
            Some(request) => request,
            None => {
                return Err(HarnessError::WorkerInvariant {
                    worker,
                    detail: "the request index was out of bounds".to_owned(),
                });
            }
        };
        let started = Instant::now();
        let outcome = executor.execute(request);
        samples.push(Sample {
            index,
            latency_ns: duration_ns(started.elapsed()),
            outcome,
        });
    }
    Ok(samples)
}

fn request_for<'a>(
    index: u64,
    workload: &'a WorkloadConfig,
    schedule: &[usize],
) -> Option<&'a RequestSpec> {
    let schedule_length = u64::try_from(schedule.len()).ok()?;
    if schedule_length == 0 {
        return None;
    }
    let schedule_index = usize::try_from(index % schedule_length).ok()?;
    let request_index = *schedule.get(schedule_index)?;
    workload.requests.get(request_index)
}
