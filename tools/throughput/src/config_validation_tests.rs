use std::path::Path;

use crate::{
    config::{RequestSpec, WorkloadConfig},
    config_validation::validate_workload,
};

fn workload(path: &str, concurrency: usize, response_bytes: usize) -> WorkloadConfig {
    WorkloadConfig {
        schema_version: 1,
        name: "bounds".to_owned(),
        iterations: 1,
        warmup_iterations: 0,
        concurrency,
        timeout_ms: 100,
        max_response_bytes: response_bytes,
        requests: vec![RequestSpec {
            name: "request".to_owned(),
            method: "GET".to_owned(),
            path: path.to_owned(),
            expected_status: 200,
            weight: 1,
            fixture_work_units: 1,
        }],
    }
}

#[test]
fn rejects_control_bytes_in_request_targets() {
    for path in ["/tab\there", "/nul\0here", "/delete\u{7f}here"] {
        assert!(validate_workload(Path::new("<test>"), &workload(path, 1, 1_024)).is_err());
    }
}

#[test]
fn rejects_aggregate_response_memory_above_fixed_cap() {
    let too_large = workload("/", 64, 16 * 1024 * 1024);
    assert!(validate_workload(Path::new("<test>"), &too_large).is_err());
    let bounded = workload("/", 4, 16 * 1024 * 1024);
    assert!(validate_workload(Path::new("<test>"), &bounded).is_ok());
}
