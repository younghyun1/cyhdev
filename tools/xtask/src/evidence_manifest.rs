//! Machine-validated W3/W8 command registration and evidence requirements.

use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use crate::{TaskError, TaskResult};

const MANIFEST_PATH: &str = "tools/final-review/evidence.manifest";
const MAX_MANIFEST_BYTES: u64 = 128 * 1024;
const MAX_REGISTRATION_SOURCE_BYTES: u64 = 2 * 1024 * 1024;
const MAX_ENTRIES: usize = 256;

enum Entry {
    Registration {
        label: String,
        path: PathBuf,
        marker: String,
    },
    Evidence {
        runtime: bool,
        label: String,
        path: PathBuf,
        minimum_bytes: u64,
        maximum_bytes: u64,
    },
}

pub(crate) fn run(root: &Path) -> TaskResult<()> {
    let manifest_path = root.join(MANIFEST_PATH);
    let manifest = read_bounded(&manifest_path, MAX_MANIFEST_BYTES)?;
    let entries = parse_manifest(&manifest)?;
    let mut registrations = 0usize;
    let mut source_evidence = 0usize;
    let mut runtime_evidence = 0usize;
    for entry in &entries {
        match entry {
            Entry::Registration { label, path, marker } => {
                validate_registration(root, label, path, marker)?;
                registrations = registrations.saturating_add(1);
            }
            Entry::Evidence {
                runtime,
                label,
                path,
                minimum_bytes,
                maximum_bytes,
            } => {
                validate_evidence(root, label, path, *minimum_bytes, *maximum_bytes, *runtime)?;
                if *runtime {
                    runtime_evidence = runtime_evidence.saturating_add(1);
                } else {
                    source_evidence = source_evidence.saturating_add(1);
                }
            }
        }
    }
    if registrations == 0 || source_evidence == 0 || runtime_evidence == 0 {
        return Err(TaskError(
            "evidence manifest must contain registrations, source evidence, and runtime evidence"
                .to_owned(),
        ));
    }
    write_receipt(
        root,
        &manifest,
        registrations,
        source_evidence,
        runtime_evidence,
    )
}

fn parse_manifest(contents: &str) -> TaskResult<Vec<Entry>> {
    let mut schema_seen = false;
    let mut entries = Vec::new();
    for (index, raw_line) in contents.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        match fields.as_slice() {
            ["schema", "1"] if !schema_seen => schema_seen = true,
            ["registration", label, path, marker] => entries.push(Entry::Registration {
                label: required_field(label, index)?,
                path: safe_relative_path(path, index)?,
                marker: required_field(marker, index)?,
            }),
            [kind @ ("source" | "runtime"), label, path, minimum, maximum] => {
                let minimum_bytes = parse_bound(minimum, index)?;
                let maximum_bytes = parse_bound(maximum, index)?;
                if minimum_bytes > maximum_bytes {
                    return Err(line_error(index, "minimum bytes exceed maximum bytes"));
                }
                entries.push(Entry::Evidence {
                    runtime: *kind == "runtime",
                    label: required_field(label, index)?,
                    path: safe_relative_path(path, index)?,
                    minimum_bytes,
                    maximum_bytes,
                });
            }
            _ => return Err(line_error(index, "invalid evidence manifest record")),
        }
        if entries.len() > MAX_ENTRIES {
            return Err(TaskError(format!(
                "evidence manifest exceeds {MAX_ENTRIES} entries"
            )));
        }
    }
    if !schema_seen {
        return Err(TaskError("evidence manifest schema 1 is missing".to_owned()));
    }
    Ok(entries)
}

fn validate_registration(
    root: &Path,
    label: &str,
    relative: &Path,
    marker: &str,
) -> TaskResult<()> {
    let source = read_bounded(&root.join(relative), MAX_REGISTRATION_SOURCE_BYTES)?;
    let occurrences = source.match_indices(marker).count();
    if occurrences == 1 {
        Ok(())
    } else {
        Err(TaskError(format!(
            "registration `{label}` marker occurs {occurrences} times in {}",
            relative.display()
        )))
    }
}

fn validate_evidence(
    root: &Path,
    label: &str,
    relative: &Path,
    minimum_bytes: u64,
    maximum_bytes: u64,
    runtime: bool,
) -> TaskResult<()> {
    let metadata = fs::symlink_metadata(root.join(relative)).map_err(|error| {
        TaskError(format!(
            "required evidence `{label}` at {} is unavailable: {error}",
            relative.display()
        ))
    })?;
    if !metadata.is_file() || metadata.len() < minimum_bytes || metadata.len() > maximum_bytes {
        return Err(TaskError(format!(
            "required evidence `{label}` at {} violates its regular-file size bound",
            relative.display()
        )));
    }
    if runtime {
        let contents = read_bounded(&root.join(relative), maximum_bytes)?;
        let first = contents.bytes().find(|byte| !byte.is_ascii_whitespace());
        if !matches!(first, Some(b'[' | b'{')) {
            return Err(TaskError(format!(
                "runtime evidence `{label}` is not a JSON document"
            )));
        }
    }
    Ok(())
}

fn read_bounded(path: &Path, maximum_bytes: u64) -> TaskResult<String> {
    let metadata = fs::metadata(path)
        .map_err(|error| TaskError(format!("failed to inspect {}: {error}", path.display())))?;
    if !metadata.is_file() || metadata.len() > maximum_bytes {
        return Err(TaskError(format!(
            "{} exceeds its {maximum_bytes}-byte input bound or is not a file",
            path.display()
        )));
    }
    fs::read_to_string(path)
        .map_err(|error| TaskError(format!("failed to read {}: {error}", path.display())))
}

fn write_receipt(
    root: &Path,
    manifest: &str,
    registrations: usize,
    source_evidence: usize,
    runtime_evidence: usize,
) -> TaskResult<()> {
    let directory = root.join("target/final-review");
    fs::create_dir_all(&directory).map_err(|error| {
        TaskError(format!("failed to create {}: {error}", directory.display()))
    })?;
    let digest = fnv1a64(manifest.as_bytes());
    let receipt = format!(
        "{{\n  \"schema_version\": 1,\n  \"manifest_digest\": \"fnv1a64:{digest:016x}\",\n  \"registrations\": {registrations},\n  \"source_evidence\": {source_evidence},\n  \"runtime_evidence\": {runtime_evidence}\n}}\n"
    );
    let path = directory.join("evidence.json");
    fs::write(&path, receipt)
        .map_err(|error| TaskError(format!("failed to write {}: {error}", path.display())))
}

fn safe_relative_path(value: &str, index: usize) -> TaskResult<PathBuf> {
    let path = PathBuf::from(value);
    let safe = !path.as_os_str().is_empty()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)));
    if safe {
        Ok(path)
    } else {
        Err(line_error(index, "evidence path is not a safe relative path"))
    }
}

fn required_field(value: &str, index: usize) -> TaskResult<String> {
    if value.trim().is_empty() {
        Err(line_error(index, "required field is empty"))
    } else {
        Ok(value.to_owned())
    }
}

fn parse_bound(value: &str, index: usize) -> TaskResult<u64> {
    value
        .parse::<u64>()
        .map_err(|_source| line_error(index, "invalid byte bound"))
}

fn line_error(index: usize, detail: &str) -> TaskError {
    TaskError(format!(
        "evidence manifest line {}: {detail}",
        index.saturating_add(1)
    ))
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut digest = 0xcbf2_9ce4_8422_2325u64;
    for byte in bytes {
        digest ^= u64::from(*byte);
        digest = digest.wrapping_mul(0x0000_0100_0000_01b3);
    }
    digest
}

#[cfg(test)]
mod tests {
    use super::parse_manifest;

    #[test]
    fn parses_bounded_registration_and_evidence_records() -> Result<(), String> {
        let manifest = "schema\t1\nregistration\tcommand:test\ttools/a.rs\tmarker\nsource\ttest-source\tsrc/a.rs\t1\t10\nruntime\ttest-report\ttarget/a.json\t2\t20\n";
        let entries = parse_manifest(manifest).map_err(|error| error.to_string())?;
        if entries.len() == 3 {
            Ok(())
        } else {
            Err(format!("expected three entries, got {}", entries.len()))
        }
    }

    #[test]
    fn rejects_parent_path_components() {
        let manifest = "schema\t1\nsource\ttest\t../secret\t1\t10\n";
        assert!(parse_manifest(manifest).is_err());
    }
}
