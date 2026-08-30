use std::path::Path;

use super::throughput_harness_command;

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
