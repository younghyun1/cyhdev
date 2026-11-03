//! PostgreSQL integration and migration-rollback commands.
//!
//! Both require `TEST_DATABASE_URL` to name the isolated maintenance database; each
//! case creates and force-drops its own databases on that server. Every case lives
//! in the backend's single `integration` test binary and is selected by module
//! path. The build uses the unit suite's package and feature selection, so after
//! `cargo xtask unit` these commands run the existing binary without relinking.

use std::{path::Path, process::Command};

use crate::{TaskResult, review::NATIVE_TEST_SELECTION, run_command};

/// Backend integration test target, `rust-be-template/tests/integration/main.rs`.
const INTEGRATION_TARGET: &str = "integration";

/// Module prefix shared by every PostgreSQL case.
const POSTGRES_CASES: &str = "postgres::";

/// Whole-chain rollback cases, skipped by `db-integration` because they must run
/// serially with the rest of the rollback coverage.
const ROLLBACK_CASES: &[&str] = &[
    "postgres::minecraft_waypoints::markers::waypoint_rollback_preserves_annotations_until_explicitly_removed",
    "postgres::account_boundaries::embedded_migration_chain_reverts_and_reapplies",
    "postgres::account_lifecycle::account_lifecycle_migration_reverts_and_reapplies",
];

/// Modules whose every case rewinds migrations.
const ROLLBACK_MODULES: &[&str] = &[
    "postgres::migration_guards::",
    "postgres::account_migrations::",
];

/// `cargo test` arguments up to and including the libtest separator.
fn integration_test_args() -> Vec<&'static str> {
    let mut args = vec!["test", "--locked"];
    args.extend(NATIVE_TEST_SELECTION);
    args.extend(["--test", INTEGRATION_TARGET, "--"]);
    args
}

pub(crate) fn run_database_integration(root: &Path) -> TaskResult<()> {
    crate::test_database::validate()?;
    let mut args = integration_test_args();
    args.extend(["--ignored", POSTGRES_CASES]);
    for skipped in ROLLBACK_CASES.iter().chain(ROLLBACK_MODULES) {
        args.extend(["--skip", skipped]);
    }
    run_command(Command::new("cargo").args(args).current_dir(root))
}

pub(crate) fn run_migration_rollback(root: &Path) -> TaskResult<()> {
    crate::test_database::validate()?;
    // Two runs of the same binary: `--exact` keeps the named cases from
    // matching longer names, but it would also turn the module prefixes into
    // exact names that match nothing.
    let mut cases = integration_test_args();
    cases.extend(["--ignored", "--exact", "--test-threads=1"]);
    cases.extend(ROLLBACK_CASES);
    run_command(Command::new("cargo").args(cases).current_dir(root))?;
    let mut modules = integration_test_args();
    modules.extend(["--ignored", "--test-threads=1"]);
    modules.extend(ROLLBACK_MODULES);
    run_command(Command::new("cargo").args(modules).current_dir(root))
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeSet,
        fs,
        path::{Path, PathBuf},
    };

    use super::{POSTGRES_CASES, ROLLBACK_CASES, ROLLBACK_MODULES};

    fn postgres_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../rust-be-template/tests/integration/postgres")
    }

    /// Module names declared by `mod name;` lines in a directory's `mod.rs`.
    fn declared_modules(dir: &Path) -> std::io::Result<BTreeSet<String>> {
        Ok(fs::read_to_string(dir.join("mod.rs"))?
            .lines()
            .filter_map(|line| line.trim().strip_prefix("mod ")?.strip_suffix(';'))
            .map(str::to_owned)
            .collect())
    }

    /// Asserts that every module file and directory under `dir` is declared,
    /// recursing into directory modules.
    fn assert_modules_declared(dir: &Path) -> std::io::Result<()> {
        let mut present = BTreeSet::new();
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if entry.file_type()?.is_dir() {
                assert_modules_declared(&entry.path())?;
                present.insert(name);
            } else if let Some(stem) = name.strip_suffix(".rs")
                && stem != "mod"
            {
                present.insert(stem.to_owned());
            }
        }
        assert_eq!(
            declared_modules(dir)?,
            present,
            "modules on disk and in {}/mod.rs differ",
            dir.display()
        );
        Ok(())
    }

    /// Rust compiles only declared modules, so a case file missing from its
    /// `mod.rs` would silently never run.
    #[test]
    fn every_postgres_module_is_declared() -> std::io::Result<()> {
        assert_modules_declared(&postgres_dir())
    }

    /// libtest passes when a filter matches nothing, so a renamed rollback case
    /// or module would otherwise drop out of both database commands unnoticed.
    #[test]
    fn rollback_selections_name_existing_cases() -> std::io::Result<()> {
        for case in ROLLBACK_CASES {
            let Some((module, function)) = case
                .strip_prefix(POSTGRES_CASES)
                .and_then(|path| path.rsplit_once("::"))
            else {
                return Err(std::io::Error::other(format!(
                    "{case} is not a postgres case path"
                )));
            };
            let source = postgres_dir().join(format!("{}.rs", module.replace("::", "/")));
            assert!(
                fs::read_to_string(&source)?.contains(&format!("async fn {function}(")),
                "{case} names no test in {}",
                source.display()
            );
        }
        let declared = declared_modules(&postgres_dir())?;
        for module in ROLLBACK_MODULES {
            let name = module
                .strip_prefix(POSTGRES_CASES)
                .and_then(|path| path.strip_suffix("::"));
            assert!(
                name.is_some_and(|name| declared.contains(name)),
                "{module} names no declared module"
            );
        }
        let unique: BTreeSet<&&str> = ROLLBACK_CASES.iter().chain(ROLLBACK_MODULES).collect();
        assert_eq!(unique.len(), ROLLBACK_CASES.len() + ROLLBACK_MODULES.len());
        Ok(())
    }
}
