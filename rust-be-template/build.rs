use std::{
    env,
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

use serde_json::Value;

const SOURCE_DATE_EPOCH: &str = "SOURCE_DATE_EPOCH";
const DEFAULT_SOURCE_DATE_EPOCH: i64 = 0;

fn main() -> ExitCode {
    match generate_build_info() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("build metadata generation failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn generate_build_info() -> Result<(), String> {
    let out_dir = PathBuf::from(required_env("OUT_DIR")?);
    let manifest_dir = PathBuf::from(required_env("CARGO_MANIFEST_DIR")?);
    let manifest_path = manifest_dir.join("Cargo.toml");
    let package_name = required_env("CARGO_PKG_NAME")?;
    let package_version = required_env("CARGO_PKG_VERSION")?;
    let rust_version = rustc_version()?;
    let metadata = dependency_metadata(&manifest_path)?;
    let build_time = build_time_utc()?;
    let generated = render_build_info(
        &package_name,
        &package_version,
        &build_time,
        &rust_version,
        &metadata.dependencies,
    )?;
    let destination = out_dir.join("build_info.rs");

    fs::write(&destination, generated).map_err(|error| {
        format!(
            "could not write generated metadata to {}: {error}",
            destination.display()
        )
    })?;

    println!("cargo:rerun-if-changed={}", manifest_path.display());
    println!(
        "cargo:rerun-if-changed={}",
        metadata.workspace_root.join("Cargo.lock").display()
    );
    println!("cargo:rerun-if-env-changed={SOURCE_DATE_EPOCH}");
    Ok(())
}

fn required_env(key: &str) -> Result<String, String> {
    match env::var(key) {
        Ok(value) => Ok(value),
        Err(error) => Err(format!(
            "required environment variable {key} is unavailable: {error}"
        )),
    }
}

fn build_time_utc() -> Result<String, String> {
    let seconds = match env::var(SOURCE_DATE_EPOCH) {
        Ok(value) => value.parse::<i64>().map_err(|error| {
            format!("{SOURCE_DATE_EPOCH} must be an integer Unix timestamp: {error}")
        })?,
        Err(env::VarError::NotPresent) => DEFAULT_SOURCE_DATE_EPOCH,
        Err(env::VarError::NotUnicode(_)) => {
            return Err(format!("{SOURCE_DATE_EPOCH} must contain valid UTF-8"));
        }
    };
    if seconds < 0 {
        return Err(format!("{SOURCE_DATE_EPOCH} must not be negative"));
    }

    match chrono::DateTime::<chrono::Utc>::from_timestamp(seconds, 0) {
        Some(timestamp) => Ok(timestamp.to_rfc3339()),
        None => Err(format!(
            "{SOURCE_DATE_EPOCH} value {seconds} is outside Chrono's supported range"
        )),
    }
}

fn rustc_version() -> Result<String, String> {
    let rustc = required_env("RUSTC")?;
    let output = Command::new(&rustc)
        .arg("--version")
        .output()
        .map_err(|error| format!("could not execute {rustc}: {error}"))?;

    if !output.status.success() {
        return Err(command_failure(&rustc, &output));
    }

    let version = String::from_utf8(output.stdout)
        .map_err(|error| format!("{rustc} returned non-UTF-8 version output: {error}"))?;
    let version = version.trim();
    if version.is_empty() {
        return Err(format!("{rustc} returned an empty version"));
    }

    Ok(version.to_owned())
}

struct DependencyMetadata {
    dependencies: Vec<LibVersion>,
    workspace_root: PathBuf,
}

struct LibVersion {
    name: String,
    version: String,
}

fn dependency_metadata(manifest_path: &Path) -> Result<DependencyMetadata, String> {
    let cargo = required_env("CARGO")?;
    let output = Command::new(&cargo)
        .args([
            "metadata",
            "--format-version",
            "1",
            "--locked",
            "--manifest-path",
        ])
        .arg(manifest_path)
        .output()
        .map_err(|error| format!("could not execute {cargo} metadata: {error}"))?;

    if !output.status.success() {
        return Err(command_failure(&format!("{cargo} metadata"), &output));
    }

    let metadata: Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("could not decode cargo metadata: {error}"))?;
    let packages = json_array(&metadata, "packages", "cargo metadata")?;
    let package = match packages.iter().find(|package| {
        package
            .get("manifest_path")
            .and_then(Value::as_str)
            .is_some_and(|candidate| paths_match(Path::new(candidate), manifest_path))
    }) {
        Some(package) => package,
        None => {
            return Err(format!(
                "cargo metadata did not contain package manifest {}",
                manifest_path.display()
            ));
        }
    };
    let package_id = json_string(package, "id", "selected package")?;
    let resolve = match metadata.get("resolve") {
        Some(resolve) if !resolve.is_null() => resolve,
        _ => return Err("cargo metadata did not contain a dependency resolution".to_owned()),
    };
    let nodes = json_array(resolve, "nodes", "cargo metadata resolution")?;
    let package_node = match nodes.iter().find(|node| {
        node.get("id")
            .and_then(Value::as_str)
            .is_some_and(|node_id| node_id == package_id)
    }) {
        Some(node) => node,
        None => {
            return Err(format!(
                "cargo metadata resolution did not contain package {package_id}"
            ));
        }
    };
    let dependency_edges = json_array(package_node, "deps", "selected package resolution")?;
    let mut dependencies = Vec::with_capacity(dependency_edges.len());

    for dependency_edge in dependency_edges {
        let dependency_id = json_string(dependency_edge, "pkg", "dependency edge")?;
        let dependency_package = match packages.iter().find(|package| {
            package
                .get("id")
                .and_then(Value::as_str)
                .is_some_and(|candidate| candidate == dependency_id)
        }) {
            Some(package) => package,
            None => {
                return Err(format!(
                    "cargo metadata did not contain dependency package {dependency_id}"
                ));
            }
        };

        dependencies.push(LibVersion {
            name: json_string(dependency_package, "name", "dependency package")?.to_owned(),
            version: json_string(dependency_package, "version", "dependency package")?.to_owned(),
        });
    }

    dependencies.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then_with(|| left.version.cmp(&right.version))
    });
    dependencies.dedup_by(|left, right| {
        left.name == right.name && left.version == right.version
    });

    let workspace_root = PathBuf::from(json_string(
        &metadata,
        "workspace_root",
        "cargo metadata",
    )?);
    Ok(DependencyMetadata {
        dependencies,
        workspace_root,
    })
}

fn json_array<'a>(value: &'a Value, key: &str, context: &str) -> Result<&'a [Value], String> {
    match value.get(key).and_then(Value::as_array) {
        Some(values) => Ok(values),
        None => Err(format!("{context} field {key:?} is not an array")),
    }
}

fn json_string<'a>(value: &'a Value, key: &str, context: &str) -> Result<&'a str, String> {
    match value.get(key).and_then(Value::as_str) {
        Some(text) => Ok(text),
        None => Err(format!("{context} field {key:?} is not a string")),
    }
}

fn paths_match(left: &Path, right: &Path) -> bool {
    if left == right {
        return true;
    }

    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
}

fn command_failure(command: &str, output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stderr = stderr.trim();
    if stderr.is_empty() {
        format!("{command} exited with {}", output.status)
    } else {
        format!("{command} exited with {}: {stderr}", output.status)
    }
}

fn render_build_info(
    package_name: &str,
    package_version: &str,
    build_time: &str,
    rust_version: &str,
    dependencies: &[LibVersion],
) -> Result<String, String> {
    let dependency_count = dependencies.len();
    let mut generated = format!(
        r#"// Generated metadata. Do not edit.
#[derive(Debug)]
pub struct LibVersion {{
    pub name: &'static str,
    pub version: &'static str,
}}

impl LibVersion {{
    pub fn get_name(&self) -> &'static str {{
        self.name
    }}

    pub fn get_version(&self) -> &'static str {{
        self.version
    }}
}}

pub struct LibVersionMap {{
    pub list: &'static [LibVersion],
}}

impl LibVersionMap {{
    pub fn get(&self, name: &str) -> Option<&LibVersion> {{
        self.list.iter().find(|version| version.get_name() == name)
    }}
}}

pub const PROJECT_NAME: &str = {package_name:?};
pub const PROJECT_VERSION: &str = {package_version:?};
pub const BUILD_TIME_UTC: &str = {build_time:?};
pub const RUSTC_VERSION: &str = {rust_version:?};
pub const LIB_VERSIONS: [LibVersion; {dependency_count}] = [
"#,
    );

    for dependency in dependencies {
        writeln!(
            generated,
            r#"    LibVersion {{
        name: {:?},
        version: {:?},
    }},"#,
            dependency.name, dependency.version
        )
        .map_err(|error| format!("could not render dependency metadata: {error}"))?;
    }

    generated.push_str(
        r#"];

pub const LIB_VERSION_MAP: LibVersionMap = LibVersionMap {
    list: &LIB_VERSIONS,
};
"#,
    );
    Ok(generated)
}
