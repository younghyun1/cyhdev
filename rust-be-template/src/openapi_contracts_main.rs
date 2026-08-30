use std::{env, path::Path, process::ExitCode};

use rust_be_template::{
    docs::ApiDoc,
    openapi_codegen::{
        error::CodegenError,
        output::{OutputMode, apply_files, generate_files},
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
    let generated = generate_files(&spec)?;
    let output = frontend_output_directory()?;
    apply_files(&output, &generated, mode)
}

fn frontend_output_directory() -> Result<std::path::PathBuf, CodegenError> {
    let backend = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = backend.parent().ok_or_else(|| {
        CodegenError::new("backend manifest has no monorepo parent directory")
    })?;
    Ok(root.join("solid-csr-spa-template/src/generated"))
}
