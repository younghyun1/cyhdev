//! PostgreSQL integration and migration-rollback commands.
//!
//! Both require `TEST_DATABASE_URL` to name the isolated maintenance database; each
//! suite creates and force-drops its own databases on that server.

use std::{path::Path, process::Command};

use crate::{TaskResult, run_command};

/// Ignored PostgreSQL suites run by `db-integration`.
const INTEGRATION_SUITES: &[&str] = &[
    "postgres_account_boundaries",
    "postgres_account_identity",
    "postgres_account_http_boundaries",
    "postgres_account_oidc",
    "postgres_account_oidc_reset",
    "postgres_account_verification",
    "postgres_account_lifecycle",
    "postgres_content_write_linearization",
    "postgres_live_chat_moderation",
    "postgres_live_chat_runtime",
    "postgres_cache_consistency",
    "postgres_i18n_sources",
    "postgres_profile_picture_history",
    "postgres_forum",
    "postgres_forum_maintenance",
    "postgres_photography_invariants",
    "postgres_wasm",
    "postgres_retention_notifications",
    "postgres_authorization_admin",
    "postgres_runtime_bounds",
    "postgres_blog_social",
    "postgres_comment_threads",
];

/// Whole-chain rollback cases, skipped by `db-integration` because they must run
/// serially with the rest of the rollback coverage.
const ROLLBACK_CASES: &[(&str, &str)] = &[
    (
        "postgres_account_boundaries",
        "embedded_migration_chain_reverts_and_reapplies",
    ),
    (
        "postgres_account_lifecycle",
        "account_lifecycle_migration_reverts_and_reapplies",
    ),
];

/// Suites whose every ignored case rewinds migrations.
const ROLLBACK_SUITES: &[&str] = &["postgres_migration_guards", "postgres_account_migrations"];

pub(crate) fn run_database_integration(root: &Path) -> TaskResult<()> {
    crate::test_database::validate()?;
    let mut args = vec!["test", "--locked", "--package", "rust-be-template"];
    for suite in INTEGRATION_SUITES {
        args.extend(["--test", suite]);
    }
    args.extend(["--no-fail-fast", "--", "--ignored"]);
    for (_, case) in ROLLBACK_CASES {
        args.extend(["--skip", case]);
    }
    run_command(Command::new("cargo").args(args).current_dir(root))
}

pub(crate) fn run_migration_rollback(root: &Path) -> TaskResult<()> {
    crate::test_database::validate()?;
    for (suite, case) in ROLLBACK_CASES {
        run_command(
            Command::new("cargo")
                .args([
                    "test",
                    "--locked",
                    "--package",
                    "rust-be-template",
                    "--test",
                    suite,
                    case,
                    "--",
                    "--ignored",
                    "--exact",
                    "--test-threads=1",
                ])
                .current_dir(root),
        )?;
    }
    let mut args = vec!["test", "--locked", "--package", "rust-be-template"];
    for suite in ROLLBACK_SUITES {
        args.extend(["--test", suite]);
    }
    args.extend(["--", "--ignored", "--test-threads=1"]);
    run_command(Command::new("cargo").args(args).current_dir(root))
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, fs, path::Path};

    use super::{INTEGRATION_SUITES, ROLLBACK_CASES, ROLLBACK_SUITES};

    /// Every `tests/postgres_*.rs` binary must be run by one of the review commands,
    /// so a new suite cannot be silently left out of verification.
    #[test]
    fn every_postgres_suite_is_registered() -> Result<(), std::io::Error> {
        let tests_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rust-be-template/tests");
        let registered: HashSet<&str> = INTEGRATION_SUITES
            .iter()
            .chain(ROLLBACK_SUITES)
            .chain(ROLLBACK_CASES.iter().map(|(suite, _)| suite))
            .copied()
            .collect();
        let mut missing = Vec::new();
        for entry in fs::read_dir(tests_dir)? {
            let name = entry?.file_name().to_string_lossy().into_owned();
            if let Some(suite) = name.strip_suffix(".rs")
                && suite.starts_with("postgres_")
                && !registered.contains(suite)
            {
                missing.push(suite.to_owned());
            }
        }
        assert!(
            missing.is_empty(),
            "unregistered PostgreSQL suites: {missing:?}"
        );
        Ok(())
    }

    #[test]
    fn suites_are_listed_once() {
        let unique: HashSet<&&str> = INTEGRATION_SUITES.iter().collect();
        assert_eq!(unique.len(), INTEGRATION_SUITES.len());
    }
}
