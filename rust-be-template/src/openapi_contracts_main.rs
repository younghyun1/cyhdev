use std::{env, path::Path, process::ExitCode};

use rust_be_template::{
    docs::ApiDoc,
    openapi_codegen::{
        error::CodegenError,
        output::{OutputMode, apply_files, generate_files},
        router_surface,
    },
};
use utoipa::OpenApi;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("openapi-contracts: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), CodegenError> {
    if env::args().nth(1).as_deref() == Some("training-surface") {
        let path = match env::args().nth(2) {
            Some(path) if env::args().count() == 3 => path,
            _ => {
                return Err(CodegenError::new(
                    "usage: openapi-contracts training-surface OUTPUT.json",
                ));
            }
        };
        let spec = serde_json::to_value(ApiDoc::openapi())?;
        router_surface::validate(&spec)?;
        let paths = match spec.get("paths").and_then(serde_json::Value::as_object) {
            Some(paths) => paths,
            None => return Err(CodegenError::new("OpenAPI has no paths")),
        };
        let mut operations = std::collections::BTreeSet::new();
        for (path, item) in paths {
            for method in [
                "get", "post", "put", "patch", "delete", "head", "options", "trace",
            ] {
                if item.get(method).is_some() {
                    operations.insert(format!("{} {path}", method.to_ascii_uppercase()));
                }
            }
        }
        let surface = serde_json::json!({"schema_version":1,"operations":operations});
        let bytes = serde_json::to_vec_pretty(&surface)?;
        return match std::fs::write(&path, bytes) {
            Ok(()) => Ok(()),
            Err(error) => Err(CodegenError::new(format!(
                "cannot write training surface: {error}"
            ))),
        };
    }
    let mode = match env::args().nth(1).as_deref() {
        Some("generate") => OutputMode::Generate,
        Some("check") => OutputMode::Check,
        Some("help" | "--help" | "-h") | None => {
            println!("Usage: openapi-contracts <generate|check>");
            return Ok(());
        }
        Some(command) => {
            return Err(CodegenError::new(format!("unknown command {command}")));
        }
    };
    let spec = serde_json::to_value(ApiDoc::openapi())?;
    router_surface::validate(&spec)?;
    let generated = generate_files(&spec)?;
    let output = frontend_output_directory()?;
    apply_files(&output, &generated, mode)
}

fn frontend_output_directory() -> Result<std::path::PathBuf, CodegenError> {
    let backend = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = backend
        .parent()
        .ok_or_else(|| CodegenError::new("backend manifest has no monorepo parent directory"))?;
    Ok(root.join("solid-csr-spa-template/src/generated"))
}
