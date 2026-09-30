//! Synthetic gates; no optimized compilation, servers or production inputs.

use std::{
    collections::BTreeSet,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

use super::{
    artifacts, config,
    coverage::{Policy, Report, Surface},
    files,
    measurement::{self, Measurement, Sample},
};

fn set(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).into()).collect()
}

fn report() -> Report {
    Report {
        schema_version: 1,
        stage: "instrumented".into(),
        binary_sha256: "binary".into(),
        campaign_sha256: "campaign".into(),
        operations: set(&["POST /api/items"]),
        pages: set(&["/items"]),
        scenarios: set(&["items.crud"]),
        failures: 0,
    }
}

#[test]
fn complete_success_coverage_accepts_matching_identity() -> Result<(), Box<dyn std::error::Error>> {
    let surface = Surface {
        schema_version: 1,
        operations: set(&["POST /api/items"]),
    };
    let policy = Policy {
        schema_version: 1,
        pages: set(&["/items"]),
        scenarios: set(&["items.crud"]),
    };
    super::coverage::check(
        &report(),
        &surface,
        &policy,
        "instrumented",
        "binary",
        "campaign",
    )?;
    Ok(())
}

#[test]
fn missing_success_and_stale_identity_fail_closed() {
    let surface = Surface {
        schema_version: 1,
        operations: set(&["POST /api/items", "DELETE /api/items/{id}"]),
    };
    let policy = Policy {
        schema_version: 1,
        pages: set(&["/items"]),
        scenarios: set(&["items.crud"]),
    };
    assert!(
        super::coverage::check(
            &report(),
            &surface,
            &policy,
            "instrumented",
            "binary",
            "campaign"
        )
        .is_err()
    );
    let mut report = report();
    report.operations.insert("DELETE /api/items/{id}".into());
    for (stage, binary, campaign) in [
        ("bolt", "binary", "campaign"),
        ("instrumented", "stale", "campaign"),
        ("instrumented", "binary", "stale"),
    ] {
        assert!(
            super::coverage::check(&report, &surface, &policy, stage, binary, campaign).is_err()
        );
    }
    report.failures = 1;
    assert!(
        super::coverage::check(
            &report,
            &surface,
            &policy,
            "instrumented",
            "binary",
            "campaign"
        )
        .is_err()
    );
}

fn measurement(stage: &str, rate: f64, latency: f64) -> Measurement {
    Measurement {
        schema_version: 1,
        stage: stage.into(),
        binary_sha256: "a".repeat(64),
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

#[test]
fn faster_candidate_accepts_but_latency_or_throughput_regression_rejects() {
    let baseline = measurement("baseline", 100.0, 10.0);
    assert!(measurement::compare(&baseline, &measurement("bolt", 110.0, 9.0), 1.05, 0.98).is_ok());
    assert!(measurement::compare(&baseline, &measurement("bolt", 97.0, 10.0), 1.05, 0.98).is_err());
    assert!(
        measurement::compare(&baseline, &measurement("bolt", 110.0, 11.0), 1.05, 0.98).is_err()
    );
}

#[test]
fn different_environment_or_invalid_samples_cannot_pass_comparison() {
    let baseline = measurement("baseline", 100.0, 10.0);
    let mut candidate = measurement("bolt", 110.0, 9.0);
    candidate.environment_sha256 = "d".repeat(64);
    assert!(measurement::compare(&baseline, &candidate, 1.05, 0.98).is_err());
    candidate.environment_sha256 = baseline.environment_sha256.clone();
    candidate.samples[0].failures = 1;
    assert!(measurement::compare(&baseline, &candidate, 1.05, 0.98).is_err());
    candidate.samples[0].failures = 0;
    candidate.samples[0].p95_ms = f64::NAN;
    assert!(measurement::compare(&baseline, &candidate, 1.05, 0.98).is_err());
    candidate.samples.clear();
    assert!(measurement::compare(&baseline, &candidate, 1.05, 0.98).is_err());
}

#[test]
fn build_names_reject_path_and_flag_injection() {
    for token in [
        "",
        "../escape",
        "cpu;touch",
        "znver3 -Cfoo",
        "a/b",
        "native\0",
    ] {
        assert!(config::token(token).is_err());
    }
    assert!(config::token("znver3").is_ok());
    assert!(config::token("campaign-01").is_ok());
}

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Result<Self, std::io::Error> {
        let path = std::env::temp_dir().join(format!(
            "cyhdev-optimization-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path)?;
        Ok(Self(path))
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.0) {
            eprintln!("synthetic fixture cleanup failed: {error}");
        }
    }
}

#[test]
fn json_inputs_are_bounded_and_digests_match_known_bytes() -> Result<(), Box<dyn std::error::Error>>
{
    let fixture = Fixture::new()?;
    let path = fixture.0.join("sample.json");
    fs::write(&path, b"null")?;
    assert!(files::read_json::<serde_json::Value>(&path, 3).is_err());
    assert_eq!(
        files::read_json::<serde_json::Value>(&path, 4)?,
        serde_json::Value::Null
    );
    fs::write(&path, b"abc")?;
    assert_eq!(
        files::digest(&path)?,
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    Ok(())
}

#[test]
fn fresh_nonempty_profiles_are_required_and_hashed() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let raw = fixture.0.join("raw/pgo");
    fs::create_dir_all(&raw)?;
    assert!(artifacts::profiles(&fixture.0, "pgo").is_err());
    fs::write(raw.join("process.profraw"), b"synthetic profile")?;
    assert_eq!(
        artifacts::profiles(&fixture.0, "pgo")?,
        ["/optimize/raw/pgo/process.profraw"]
    );
    fs::write(raw.join("empty.profraw"), b"")?;
    assert!(artifacts::profiles(&fixture.0, "pgo").is_err());
    Ok(())
}

#[cfg(unix)]
#[test]
fn profile_symlinks_cannot_escape_the_campaign() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    fs::create_dir_all(fixture.0.join("raw/pgo"))?;
    fs::write(fixture.0.join("outside"), b"synthetic")?;
    std::os::unix::fs::symlink(
        fixture.0.join("outside"),
        fixture.0.join("raw/pgo/link.profraw"),
    )?;
    assert!(artifacts::profiles(&fixture.0, "pgo").is_err());
    Ok(())
}

#[test]
fn optimization_builder_preserves_the_unstripped_host_contract() {
    let dockerfile = include_str!("../../../../rust-be-template/Dockerfile");
    assert!(dockerfile.contains("AS optimization-builder"));
    assert!(dockerfile.contains("rustup component add rust-src llvm-tools-preview"));
    assert!(dockerfile.contains("cargo fetch --locked --manifest-path"));
    let config = include_str!("../../../../tools/optimization/config.example.json");
    assert!(serde_json::from_str::<config::Config>(config).is_ok());
}

#[cfg(unix)]
#[test]
fn driver_failure_and_timeout_are_errors() {
    let mut failure = std::process::Command::new("bash");
    failure.args(["-c", "exit 17"]);
    assert!(super::process::bounded(&mut failure, 1).is_err());
    let mut timeout = std::process::Command::new("bash");
    timeout.args(["-c", "sleep 3"]);
    let start = std::time::Instant::now();
    assert!(super::process::bounded(&mut timeout, 1).is_err());
    assert!(start.elapsed() < std::time::Duration::from_secs(3));
}

#[test]
fn unfinished_example_cannot_start_optimized_compilation() -> Result<(), Box<dyn std::error::Error>>
{
    let fixture = Fixture::new()?;
    fs::create_dir_all(fixture.0.join("tools/optimization"))?;
    fs::write(
        fixture.0.join("tools/optimization/coverage.json"),
        include_str!("../../../../tools/optimization/coverage.json"),
    )?;
    let campaign = fixture.0.join("campaign.json");
    fs::write(
        &campaign,
        include_str!("../../../../tools/optimization/campaign.example.json"),
    )?;
    let surface = fixture.0.join("surface.json");
    files::write_json(
        &surface,
        &Surface {
            schema_version: 1,
            operations: set(&["GET /api/blog/posts"]),
        },
    )?;
    assert!(super::inventory::preflight(&fixture.0, &campaign, &surface).is_err());
    Ok(())
}
