//! Backend binaries are declared with `test = false`, so Cargo builds no test
//! harness for them. A binary added without that setting links another copy of
//! the backend into every `cargo test`, and a test written in a binary's sources
//! never compiles or runs; these checks catch both.

use std::{
    collections::BTreeSet,
    fs, io,
    path::{Path, PathBuf},
};

fn backend_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rust-be-template")
}

/// The `path` and `test` keys of one `[[bin]]` table.
struct BinaryTarget {
    path: Option<String>,
    test: Option<String>,
}

/// Reads every `[[bin]]` table of the backend manifest. The manifest keeps one
/// key per line, so a line scan suffices without a TOML parser.
fn declared_binaries() -> io::Result<Vec<BinaryTarget>> {
    let manifest = fs::read_to_string(backend_dir().join("Cargo.toml"))?;
    let mut binaries = Vec::new();
    let mut in_binary = false;
    for line in manifest.lines().map(str::trim) {
        if line.starts_with('[') {
            in_binary = line == "[[bin]]";
            if in_binary {
                binaries.push(BinaryTarget {
                    path: None,
                    test: None,
                });
            }
            continue;
        }
        let (true, Some(binary), Some((key, value))) =
            (in_binary, binaries.last_mut(), line.split_once('='))
        else {
            continue;
        };
        let value = value.trim().trim_matches('"').to_owned();
        match key.trim() {
            "path" => binary.path = Some(value),
            "test" => binary.test = Some(value),
            _ => {}
        }
    }
    Ok(binaries)
}

/// Binary roots Cargo would discover without a `[[bin]]` table.
fn discoverable_binaries() -> io::Result<BTreeSet<String>> {
    let mut roots = BTreeSet::new();
    if backend_dir().join("src/main.rs").is_file() {
        roots.insert("src/main.rs".to_owned());
    }
    for entry in fs::read_dir(backend_dir().join("src/bin"))? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if entry.file_type()?.is_dir() {
            if entry.path().join("main.rs").is_file() {
                roots.insert(format!("src/bin/{name}/main.rs"));
            }
        } else if name.ends_with(".rs") {
            roots.insert(format!("src/bin/{name}"));
        }
    }
    Ok(roots)
}

/// The module name of a `mod name;` line, with or without visibility.
fn file_module(line: &str) -> Option<&str> {
    let (visibility, declaration) = line.split_once("mod ")?;
    let name = declaration.strip_suffix(';')?;
    (visibility.is_empty() || visibility.starts_with("pub")).then_some(name)
}

/// `file:line` of every test attribute or `cfg(test)` gate in `root` and the
/// file modules it reaches. Resolution follows rustc: a crate root, `mod.rs`,
/// or `#[path]` file owns its directory, and any other file owns a directory
/// named after its stem. A wrong resolution fails on the missing file.
fn test_markers(root: PathBuf) -> io::Result<Vec<String>> {
    let mut markers = Vec::new();
    let mut pending = vec![(root, true)];
    while let Some((file, owns_directory)) = pending.pop() {
        let source = fs::read_to_string(&file)?;
        let (Some(directory), Some(stem)) = (file.parent(), file.file_stem()) else {
            return Err(io::Error::other(format!("{} has no stem", file.display())));
        };
        let modules = match owns_directory {
            true => directory.to_path_buf(),
            false => directory.join(stem),
        };
        let mut path_attribute = None;
        for (number, line) in source.lines().map(str::trim).enumerate() {
            if line.starts_with("#[")
                && (line.starts_with("#[test") || line.contains("::test") || line.contains("(test"))
            {
                markers.push(format!("{}:{}", file.display(), number + 1));
            }
            if let Some(path) = line
                .strip_prefix("#[path = \"")
                .and_then(|rest| rest.strip_suffix("\"]"))
            {
                path_attribute = Some(directory.join(path));
            } else if let Some(name) = file_module(line) {
                let flat = modules.join(format!("{name}.rs"));
                pending.push(match path_attribute.take() {
                    Some(path) => (path, true),
                    None if flat.is_file() => (flat, false),
                    None => (modules.join(name).join("mod.rs"), true),
                });
            }
        }
    }
    Ok(markers)
}

#[test]
fn every_backend_binary_is_declared_without_a_test_harness() -> io::Result<()> {
    let binaries = declared_binaries()?;
    let declared: BTreeSet<String> = binaries.iter().filter_map(|bin| bin.path.clone()).collect();
    assert_eq!(
        declared.len(),
        binaries.len(),
        "a [[bin]] path is missing or repeated"
    );
    let undeclared: Vec<String> = discoverable_binaries()?
        .difference(&declared)
        .cloned()
        .collect();
    assert!(
        undeclared.is_empty(),
        "declare {undeclared:?} with test = false"
    );
    for binary in &binaries {
        assert_eq!(
            binary.test.as_deref(),
            Some("false"),
            "{:?} must set test = false",
            binary.path
        );
    }
    Ok(())
}

#[test]
fn backend_binaries_hold_no_tests() -> io::Result<()> {
    // The same scan over the library must find its tests, or a broken scan
    // would pass the binaries vacuously.
    assert!(!test_markers(backend_dir().join("src/lib.rs"))?.is_empty());
    for binary in declared_binaries()? {
        let Some(path) = binary.path else { continue };
        let markers = test_markers(backend_dir().join(path))?;
        assert!(
            markers.is_empty(),
            "tests in a test = false binary: {markers:?}"
        );
    }
    Ok(())
}
