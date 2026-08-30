use std::{
    env,
    error::Error,
    fmt::Write,
    fs,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

fn main() -> Result<(), Box<dyn Error>> {
    let manifest_dir = env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .ok_or("CARGO_MANIFEST_DIR is missing")?;
    let workspace = manifest_dir
        .parent()
        .and_then(Path::parent)
        .ok_or("throughput manifest is not under the workspace tools directory")?;
    let mut inputs = vec![
        manifest_dir.join("build.rs"),
        manifest_dir.join("Cargo.toml"),
        workspace.join("Cargo.toml"),
        workspace.join("Cargo.lock"),
        workspace.join("rust-toolchain.toml"),
    ];
    collect_files(&manifest_dir.join("src"), &mut inputs)?;
    inputs.sort();

    let mut digest = Sha256::new();
    for path in inputs {
        println!("cargo:rerun-if-changed={}", path.display());
        let relative = path.strip_prefix(workspace)?;
        let name = relative.to_string_lossy();
        let bytes = fs::read(&path)?;
        digest.update((name.len() as u64).to_le_bytes());
        digest.update(name.as_bytes());
        digest.update((bytes.len() as u64).to_le_bytes());
        digest.update(bytes);
    }
    let bytes = digest.finalize();
    let mut encoded = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        write!(&mut encoded, "{byte:02x}")?;
    }
    println!("cargo:rustc-env=THROUGHPUT_IMPLEMENTATION_DIGEST=sha256:{encoded}");
    Ok(())
}

fn collect_files(directory: &Path, output: &mut Vec<PathBuf>) -> Result<(), Box<dyn Error>> {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_files(&path, output)?;
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            output.push(path);
        }
    }
    Ok(())
}
