//! Rotated benchmark rounds and the stage comparisons that decide acceptance.

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde_json::{Value, json};

use super::{campaign, config::Config, files, measurement, process, state::State};
use crate::{TaskError, TaskResult};

/// Stages every campaign compares, in the first round's order.
const STAGES: [&str; 3] = ["baseline", "pgo", "bolt"];

/// `(reference, candidate)`: each candidate must not regress against its reference.
const COMPARISONS: [(&str, &str); 3] = [("baseline", "pgo"), ("baseline", "bolt"), ("pgo", "bolt")];

/// Binaries a re-benchmark copies from an earlier run; BOLT reuses the PGO seed worker.
const BINARIES: [&str; 5] = [
    "baseline-rust-be-template",
    "pgo-rust-be-template",
    "bolt-rust-be-template",
    "baseline-minecraft-seed",
    "pgo-minecraft-seed",
];

/// Stage order for a zero-based round. Rotating by one stage per round puts
/// every stage first, second and last once in each group of three rounds, so
/// position within a round favors none of them.
pub(super) fn order(round: usize) -> [&'static str; 3] {
    let mut stages = STAGES;
    stages.rotate_left(round % STAGES.len());
    stages
}

impl State {
    /// Benchmarks every stage in rotated rounds, records per-round medians and
    /// paired ratios, and fails when a candidate exceeds its budget.
    pub(super) fn benchmark_rounds(&mut self) -> TaskResult<()> {
        self.receipt["benchmark_host"] = host_power();
        let rounds = self.config.benchmark_rounds;
        let mut summaries = Vec::with_capacity(rounds);
        for round in 1..=rounds {
            let mut summary = serde_json::Map::new();
            for stage in order(round - 1) {
                self.benchmark(stage, round)?;
                let medians = measurement::summary(&self.measurement(stage, round)?)?;
                println!(
                    "Round {round} {stage}: {:.0} requests/s, p95 {:.3} ms, p99 {:.3} ms",
                    medians.requests_per_second, medians.p95_ms, medians.p99_ms
                );
                summary.insert(stage.to_owned(), json!(medians));
            }
            summaries.push(Value::Object(summary));
        }
        self.receipt["benchmark_rounds"] = json!(summaries);
        let mut regressions = Vec::new();
        for (reference, candidate) in COMPARISONS {
            let mut pairs = Vec::with_capacity(rounds);
            for round in 1..=rounds {
                pairs.push((
                    self.measurement(reference, round)?,
                    self.measurement(candidate, round)?,
                ));
            }
            let ratios = measurement::ratios(&pairs)?;
            let accepted = measurement::within(
                &ratios,
                self.config.maximum_latency_ratio,
                self.config.minimum_throughput_ratio,
            );
            println!(
                "{candidate} vs {reference}: throughput {:.3}x, p95 {:.3}x, p99 {:.3}x (median of {rounds} paired rounds){}",
                ratios.throughput,
                ratios.p95,
                ratios.p99,
                if accepted { "" } else { ", beyond budget" }
            );
            self.receipt["comparisons"][format!("{candidate}_vs_{reference}")] =
                json!({"ratios": ratios, "accepted": accepted});
            if !accepted {
                regressions.push(format!("{candidate} vs {reference}"));
            }
        }
        self.save()?;
        if regressions.is_empty() {
            Ok(())
        } else {
            Err(TaskError(format!(
                "{} regresses beyond budgets; per-round medians are in {}",
                regressions.join(" and "),
                self.directory.join("receipt.json").display()
            )))
        }
    }
}

/// Records the CPU frequency policy and warns when it is not performance.
/// Under a balanced policy, clocks and idle states follow load, which makes
/// closed-loop loopback benchmarks noisier; rotation cancels slow drift, not
/// that noise.
fn host_power() -> Value {
    let read = |name: &str| {
        fs::read_to_string(Path::new("/sys/devices/system/cpu/cpu0/cpufreq").join(name))
            .ok()
            .map(|value| value.trim().to_owned())
    };
    let governor = read("scaling_governor");
    let preference = read("energy_performance_preference");
    // amd-pstate and intel_pstate in active mode keep the powersave governor
    // and express the performance profile through the energy preference.
    if governor.as_deref() != Some("performance") && preference.as_deref() != Some("performance") {
        println!(
            "Warning: CPU frequency policy is {} with energy preference {}; `powerprofilesctl set performance` gives steadier benchmarks",
            governor.as_deref().unwrap_or("unknown"),
            preference.as_deref().unwrap_or("unknown")
        );
    }
    json!({"governor": governor, "energy_performance_preference": preference})
}

/// Re-measures an earlier run's binaries in rotated rounds without compiling,
/// writing reports and a receipt to the fresh run directory the config names.
pub(super) fn rebenchmark(
    root: &Path,
    config_path: PathBuf,
    config: Config,
    previous: &Path,
) -> TaskResult<()> {
    let revision = process::revision(root)?;
    let previous = match previous.canonicalize() {
        Ok(path) => path,
        Err(error) => {
            return Err(TaskError(format!(
                "cannot resolve the earlier run {}: {error}",
                previous.display()
            )));
        }
    };
    if let Some(missing) = BINARIES.iter().find(|name| !previous.join(name).is_file()) {
        return Err(TaskError(format!(
            "earlier run {} has no {missing}",
            previous.display()
        )));
    }
    let directory = campaign::fresh_directory(root, &config.name)?;
    for name in BINARIES {
        if let Err(error) = fs::copy(previous.join(name), directory.join(name)) {
            return Err(TaskError(format!("cannot copy {name}: {error}")));
        }
    }
    let mut state = State {
        root: root.to_path_buf(),
        directory,
        config_digest: files::digest(&config_path)?,
        campaign_digest: files::digest(&config.campaign)?,
        config,
        config_path,
        revision,
        image: String::new(),
        receipt: json!({"schema_version":1,"status":"active","kind":"rebenchmark","source_run":previous.display().to_string(),"stages":[]}),
    };
    state.receipt["revision"] = json!(state.revision);
    state.receipt["config_sha256"] = json!(state.config_digest);
    state.receipt["campaign_sha256"] = json!(state.campaign_digest);
    for stage in STAGES {
        state.record(stage)?;
    }
    let outcome = state.benchmark_rounds();
    state.receipt["status"] = json!(if outcome.is_ok() {
        "within_budget"
    } else {
        "regressed"
    });
    state.save()?;
    outcome
}
