use std::path::Path;

use super::{eu5_web, throughput_harness_command};

fn required_index(contents: &str, needle: &str) -> usize {
    let index = contents.find(needle).unwrap_or(contents.len());
    assert!(
        index < contents.len(),
        "required build instruction is missing: {needle}"
    );
    index
}

#[test]
fn optimized_build_reports_an_uninitialized_eu5_submodule() {
    let result = eu5_web::require_checkout(Path::new("/cyhdev-test-root-without-an-eu5-submodule"));
    assert!(result.is_err());
    let Err(error) = result else { return };

    assert!(
        error
            .to_string()
            .contains("git submodule update --init --recursive")
    );
}

#[test]
fn eu5_browser_build_uses_development_profile_and_web_features() {
    let command = eu5_web::command(
        Path::new("/workspace"),
        Path::new("/workspace/public/eu5/pkg"),
    );
    let arguments = command
        .get_args()
        .map(|argument| argument.to_string_lossy().into_owned())
        .collect::<Vec<_>>();

    assert_eq!(
        arguments,
        [
            "build",
            "/workspace/vendor/eu5-location-filter",
            "--dev",
            "--target",
            "web",
            "--no-pack",
            "--no-typescript",
            "--out-dir",
            "/workspace/public/eu5/pkg",
            "--",
            "--locked",
            "--no-default-features",
            "--features",
            "web"
        ]
    );
    assert_eq!(command.get_current_dir(), Some(Path::new("/workspace")));
}

#[test]
fn throughput_harness_uses_the_release_profile() {
    let command = throughput_harness_command(Path::new("/workspace"), &[]);
    let arguments = command
        .get_args()
        .map(|argument| argument.to_string_lossy().into_owned())
        .collect::<Vec<_>>();

    assert_eq!(
        arguments,
        [
            "run",
            "--locked",
            "--release",
            "--package",
            "throughput-harness"
        ]
    );
    assert_eq!(command.get_current_dir(), Some(Path::new("/workspace")));
}

#[test]
fn throughput_harness_forwards_arguments_after_cargo_options() {
    let forwarded = ["record".to_owned(), "--target".to_owned()];
    let command = throughput_harness_command(Path::new("/workspace"), &forwarded);
    let arguments = command
        .get_args()
        .map(|argument| argument.to_string_lossy().into_owned())
        .collect::<Vec<_>>();

    assert_eq!(
        arguments,
        [
            "run",
            "--locked",
            "--release",
            "--package",
            "throughput-harness",
            "--",
            "record",
            "--target"
        ]
    );
}

#[test]
fn container_build_epochs_follow_stable_dependency_layers() {
    let dockerfile = include_str!("../../../rust-be-template/Dockerfile");
    let frontend_start = required_index(dockerfile, " AS frontend");
    let host_start = required_index(dockerfile, " AS host-release");
    let frontend = &dockerfile[frontend_start..host_start];
    assert!(
        required_index(frontend, "RUN --mount=type=cache,id=cyhdev-npm-cache")
            < required_index(frontend, "ARG APP_BUILD_EPOCH")
    );

    let artifact_start = required_index(dockerfile, "FROM scratch AS artifact");
    let host = &dockerfile[host_start..artifact_start];
    assert!(
        required_index(host, "rustup component add rust-src")
            < required_index(host, "ARG APP_BUILD_EPOCH")
    );
    assert!(
        required_index(host, "COPY --from=frontend") < required_index(host, "ARG APP_BUILD_EPOCH")
    );
}

#[test]
fn container_build_persists_expensive_package_caches() {
    let dockerfile = include_str!("../../../rust-be-template/Dockerfile");
    for cache_id in [
        "id=cyhdev-eu5-target",
        "id=cyhdev-eu5-cargo-registry",
        "id=cyhdev-eu5-cargo-git",
        "id=cyhdev-wasm-pack-cache",
        "id=cyhdev-npm-cache",
    ] {
        assert!(
            dockerfile.contains(cache_id),
            "missing cache mount {cache_id}"
        );
    }
    assert!(
        include_str!("../../../.dockerignore")
            .lines()
            .any(|line| line == "**/logs")
    );
}
