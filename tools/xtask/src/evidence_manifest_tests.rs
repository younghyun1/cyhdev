//! Exercise checked-in registrations without requiring ignored runtime reports.

use std::{fs, path::Path};

use super::{
    Entry, MANIFEST_PATH, clear_runtime_receipts, parse_manifest, validate_evidence,
    validate_registration,
};

#[test]
fn checked_in_registrations_and_source_evidence_are_current()
-> Result<(), Box<dyn std::error::Error>> {
    let root = crate::workspace_root()?;
    let manifest = fs::read_to_string(root.join(MANIFEST_PATH))?;
    let mut registrations = 0;
    let mut sources = 0;
    let mut runtime = 0;
    for entry in parse_manifest(&manifest)? {
        match entry {
            Entry::Registration {
                label,
                path,
                marker,
            } => {
                validate_registration(&root, &label, &path, &marker)?;
                registrations += 1;
            }
            Entry::Evidence {
                runtime: false,
                label,
                path,
                minimum_bytes,
                maximum_bytes,
            } => {
                validate_evidence(&root, &label, &path, minimum_bytes, maximum_bytes, false)?;
                sources += 1;
            }
            Entry::Evidence { runtime: true, .. } => runtime += 1,
        }
    }
    assert!(registrations > 0 && sources > 0 && runtime > 0);
    Ok(())
}

#[test]
fn parses_bounded_registration_and_evidence_records() -> crate::TaskResult<()> {
    let manifest = "schema\t1\nregistration\tcommand:test\ttools/a.rs\tmarker\nsource\ttest-source\tsrc/a.rs\t1\t10\nruntime\ttest-report\ttarget/a.json\t2\t20\n";
    assert_eq!(parse_manifest(manifest)?.len(), 3);
    Ok(())
}

#[test]
fn rejects_parent_paths_and_runtime_paths_outside_target() {
    for manifest in [
        "schema\t1\nsource\ttest\t../secret\t1\t10\n",
        "schema\t1\nruntime\ttest\tsrc/a.rs\t1\t10\n",
    ] {
        assert!(parse_manifest(manifest).is_err());
    }
}

#[test]
fn full_review_removes_previous_runtime_receipts_only() -> Result<(), Box<dyn std::error::Error>> {
    let directory = std::env::temp_dir().join(format!("cyhdev-evidence-{}", std::process::id()));
    fs::create_dir(&directory)?;
    let result = check_receipt_cleanup(&directory);
    let cleanup = fs::remove_dir_all(&directory);
    result?;
    cleanup?;
    Ok(())
}

fn check_receipt_cleanup(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(root.join("tools/final-review"))?;
    fs::create_dir_all(root.join("target/final-review"))?;
    fs::write(root.join("source.rs"), "fixture")?;
    fs::write(root.join("target/report.json"), "[]")?;
    fs::write(root.join("target/final-review/evidence.json"), "{}")?;
    fs::write(
        root.join(MANIFEST_PATH),
        "schema\t1\nsource\tfixture\tsource.rs\t1\t100\nruntime\tfixture\ttarget/report.json\t2\t100\n",
    )?;
    clear_runtime_receipts(root)?;
    assert!(!root.join("target/report.json").exists());
    assert!(!root.join("target/final-review/evidence.json").exists());
    assert_eq!(fs::read_to_string(root.join("source.rs"))?, "fixture");
    // Starting again after missing receipts is safe; failed producers leave
    // those paths absent and the normal evidence check rejects them.
    clear_runtime_receipts(root)?;
    assert!(
        validate_evidence(
            root,
            "fixture",
            Path::new("target/report.json"),
            2,
            100,
            true
        )
        .is_err()
    );
    Ok(())
}
