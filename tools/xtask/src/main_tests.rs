use std::path::Path;

use super::{eu5_web, throughput_harness_command};

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
