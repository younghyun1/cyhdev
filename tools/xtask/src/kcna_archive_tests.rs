use std::path::Path;

use super::kcna_archive::{push_command, snapshot_command, validate_destination};

fn arguments(command: &std::process::Command) -> Vec<String> {
    command
        .get_args()
        .map(|argument| argument.to_string_lossy().into_owned())
        .collect()
}

#[test]
fn snapshot_runs_the_collector_in_the_development_profile() {
    let forwarded = ["--rebuild".to_owned()];
    let command = snapshot_command(Path::new("/workspace"), &forwarded);
    let arguments = arguments(&command);

    assert_eq!(
        arguments,
        [
            "run",
            "--locked",
            "--package",
            "kcna-archive",
            "--",
            "--snapshot-dir",
            "/workspace/rust-be-template/data/kcna_archive",
            "database",
            "snapshot",
            "--rebuild"
        ]
    );
    assert!(!arguments.iter().any(|argument| argument == "--release"));
    assert_eq!(command.get_current_dir(), Some(Path::new("/workspace")));
}

#[test]
fn push_copies_directory_contents_and_swaps_at_the_end() {
    let command = push_command(
        Path::new("/workspace/rust-be-template/data/kcna_archive"),
        "host:dist/data/kcna_archive/",
    );
    let arguments = arguments(&command);

    assert_eq!(command.get_program(), "rsync");
    assert!(arguments.contains(&"--delay-updates".to_owned()));
    assert!(arguments.contains(&"--delete-delay".to_owned()));
    assert_eq!(
        arguments
            .get(arguments.len().saturating_sub(2))
            .map(String::as_str),
        Some("/workspace/rust-be-template/data/kcna_archive/")
    );
    assert_eq!(
        arguments.last().map(String::as_str),
        Some("host:dist/data/kcna_archive/")
    );
}

#[test]
fn destinations_that_rsync_or_a_shell_could_misread_are_rejected() {
    for accepted in [
        "host:dist/data/kcna_archive/",
        "user@host.example:~/dist/data/kcna_archive/",
        "/srv/site/data/kcna_archive/",
    ] {
        assert!(validate_destination(accepted).is_ok(), "{accepted}");
    }
    for rejected in [
        "",
        "--rsh=sh",
        "host:dist/data/kcna_archive",
        "host:dist/$(id)/",
        "host:dist/a b/",
        "host:dist/;rm/",
    ] {
        assert!(validate_destination(rejected).is_err(), "{rejected}");
    }
}
