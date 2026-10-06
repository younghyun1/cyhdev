//! Compiler freshness and bounded lookup checks; default tests never use the network.

use super::nightly;

#[test]
fn nightly_date_is_stable_and_ignores_package_dates() -> crate::TaskResult<()> {
    let manifest =
        "manifest-version = \"2\"\ndate = \"2026-10-06\"\n\n[pkg.rust]\ndate = \"2026-10-04\"\n";
    assert_eq!(nightly::from_manifest(manifest)?, "nightly-2026-10-06");
    assert_eq!(
        nightly::from_manifest(manifest)?,
        nightly::from_manifest(manifest)?
    );
    let next = manifest.replace("2026-10-06", "2026-10-07");
    assert_ne!(
        nightly::from_manifest(manifest)?,
        nightly::from_manifest(&next)?
    );
    Ok(())
}

#[test]
fn malformed_missing_or_duplicate_dates_fail_without_fallback() {
    for date in [
        "",
        "2026-13-01",
        "2026-00-01",
        "2026-10-00",
        "2026-04-31",
        "2026-02-29",
        "2100-02-29",
        "2026-1-06",
        "2026-10-06;touch",
        "２０２６-10-06",
    ] {
        let manifest = format!("manifest-version = \"2\"\ndate = \"{date}\"\n");
        assert!(
            nightly::from_manifest(&manifest).is_err(),
            "accepted {date}"
        );
    }
    for manifest in [
        "manifest-version = \"2\"\n[pkg.rust]\ndate = \"2026-10-06\"",
        "manifest-version = \"3\"\ndate = \"2026-10-06\"",
        "date = \"2026-10-06\"",
        "manifest-version = \"2\"\ndate = \"2026-10-06\"\ndate = \"2026-10-07\"",
    ] {
        assert!(nightly::from_manifest(manifest).is_err());
    }
    assert!(nightly::from_manifest(&"x".repeat(2 * 1024 * 1024 + 1)).is_err());
}

#[test]
fn real_calendar_dates_and_line_endings_are_supported() -> crate::TaskResult<()> {
    for date in ["2024-02-29", "2400-02-29", "2026-12-31"] {
        let manifest = format!("manifest-version = \"2\"\r\ndate = \"{date}\"\r\n[pkg.rust]\r\n");
        assert_eq!(
            nightly::from_manifest(&manifest)?,
            format!("nightly-{date}")
        );
    }
    Ok(())
}

#[test]
fn lookup_is_bounded_and_requests_fresh_https_metadata() {
    let command = nightly::manifest_command();
    let arguments: Vec<_> = command.get_args().collect();
    for (flag, value) in [
        ("--max-time", "30"),
        ("--max-filesize", "2097152"),
        ("--proto", "=https"),
        ("--proto-redir", "=https"),
        ("--header", "Cache-Control: no-cache"),
    ] {
        assert!(arguments.windows(2).any(|pair| pair == [flag, value]));
    }
    assert!(arguments.contains(&std::ffi::OsStr::new("--fail")));
}

#[test]
#[ignore = "explicit read-only check of the official Rust nightly distribution manifest"]
fn published_nightly_manifest_resolves() -> crate::TaskResult<()> {
    let manifest = super::process::output(&mut nightly::manifest_command())?;
    let toolchain = nightly::from_manifest(&manifest)?;
    println!("Latest published nightly: {toolchain}");
    Ok(())
}
