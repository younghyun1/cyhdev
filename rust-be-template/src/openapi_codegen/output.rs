use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use serde_json::Value;

use super::{
    client::{render_client, validate_method_names},
    error::CodegenError,
    model::{Operation, select_contract},
    naming::{client_factory_name, type_file_name},
    runtime::CLIENT_RUNTIME,
    surface::FRONTEND_OPERATIONS,
    type_modules::{render_api_response, render_component, render_type_index},
    typescript::GENERATED_HEADER,
};

/// Whether generated artifacts should be written or compared with disk.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputMode {
    Generate,
    Check,
}

/// Complete generated frontend contract artifact set, keyed by relative path.
#[derive(Debug)]
pub struct GeneratedFiles {
    pub files: BTreeMap<PathBuf, String>,
}

/// Generates every frontend-consumed contract from the backend OpenAPI document.
pub fn generate_files(spec: &Value) -> Result<GeneratedFiles, CodegenError> {
    let contract = select_contract(spec, FRONTEND_OPERATIONS)?;
    if contract.operations.is_empty() {
        return Err(CodegenError::new(
            "OpenAPI contains no frontend contract operations",
        ));
    }
    validate_method_names(&contract.operations)?;

    let groups = group_operations(&contract.operations);
    let mut files = BTreeMap::new();
    insert_file(&mut files, "index.ts", render_root_index())?;
    insert_file(&mut files, "api-client.ts", render_client_index(&groups))?;
    insert_file(&mut files, "api-types.ts", render_api_types_index())?;
    insert_file(
        &mut files,
        "runtime.ts",
        format!("{GENERATED_HEADER}{CLIENT_RUNTIME}"),
    )?;

    for (group, operations) in &groups {
        insert_file(
            &mut files,
            format!("clients/{group}.ts"),
            render_client(operations, &client_factory_name(group))?,
        )?;
    }

    insert_file(&mut files, "types/api-response.ts", render_api_response())?;
    insert_file(
        &mut files,
        "types/index.ts",
        render_type_index(&contract.components),
    )?;
    for (name, schema) in &contract.components {
        insert_file(
            &mut files,
            format!("types/{}.ts", type_file_name(name)),
            render_component(name, schema)?,
        )?;
    }

    Ok(GeneratedFiles { files })
}

fn group_operations(operations: &[Operation]) -> BTreeMap<String, Vec<Operation>> {
    let mut groups: BTreeMap<String, Vec<Operation>> = BTreeMap::new();
    for operation in operations {
        groups
            .entry(operation.client_group.clone())
            .or_default()
            .push(operation.clone());
    }
    groups
}

fn render_client_index(groups: &BTreeMap<String, Vec<Operation>>) -> String {
    let mut output = String::from(GENERATED_HEADER);
    output.push_str("import type { ApiTransport } from \"./runtime\";\n");
    for group in groups.keys() {
        let factory = client_factory_name(group);
        output.push_str("import { ");
        output.push_str(&factory);
        output.push_str(" } from \"./clients/");
        output.push_str(group);
        output.push_str("\";\n");
    }
    output.push_str(
        "\nexport { ApiContractError } from \"./runtime\";\n\
         export type { ApiRequestOptions, ApiTransport } from \"./runtime\";\n\n\
         export function createApiClient(transport: ApiTransport) {\n\
         \x20 return {\n",
    );
    for group in groups.keys() {
        output.push_str("    ...");
        output.push_str(&client_factory_name(group));
        output.push_str("(transport),\n");
    }
    output.push_str(
        "  } as const;\n\
         }\n\n\
         export type ApiClient = ReturnType<typeof createApiClient>;\n",
    );
    output
}

fn render_api_types_index() -> String {
    format!("{GENERATED_HEADER}export * from \"./types\";\n")
}

fn render_root_index() -> String {
    format!(
        "{GENERATED_HEADER}export * from \"./api-client\";\nexport * from \"./api-types\";\n"
    )
}

fn insert_file(
    files: &mut BTreeMap<PathBuf, String>,
    path: impl Into<PathBuf>,
    contents: String,
) -> Result<(), CodegenError> {
    let path = path.into();
    if files.insert(path.clone(), contents).is_some() {
        Err(CodegenError::new(format!(
            "generated output path collides: {}",
            path.display()
        )))
    } else {
        Ok(())
    }
}

/// Writes generated files or fails when checked-in files have drifted.
pub fn apply_files(
    output_directory: &Path,
    generated: &GeneratedFiles,
    mode: OutputMode,
) -> Result<(), CodegenError> {
    if mode == OutputMode::Generate {
        fs::create_dir_all(output_directory)?;
    }
    let mut drifted = Vec::new();
    for (relative_path, contents) in &generated.files {
        let path = output_directory.join(relative_path);
        match mode {
            OutputMode::Generate => write_if_changed(&path, contents)?,
            OutputMode::Check => match fs::read_to_string(&path) {
                Ok(current) if current == *contents => {}
                Ok(_) => drifted.push(relative_path.display().to_string()),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    drifted.push(relative_path.display().to_string());
                }
                Err(error) => return Err(error.into()),
            },
        }
    }

    let expected: BTreeSet<PathBuf> = generated.files.keys().cloned().collect();
    let stale: Vec<PathBuf> = collect_typescript_files(output_directory)?
        .difference(&expected)
        .cloned()
        .collect();
    match mode {
        OutputMode::Generate => {
            for relative_path in stale {
                fs::remove_file(output_directory.join(relative_path))?;
            }
        }
        OutputMode::Check => {
            drifted.extend(stale.iter().map(|path| path.display().to_string()));
        }
    }

    if drifted.is_empty() {
        Ok(())
    } else {
        drifted.sort();
        Err(CodegenError::new(format!(
            "generated frontend contracts have drifted: {}; from the repository root run `cargo run --locked --package rust-be-template --bin openapi-contracts -- generate`",
            drifted.join(", ")
        )))
    }
}

fn collect_typescript_files(directory: &Path) -> Result<BTreeSet<PathBuf>, CodegenError> {
    let mut files = BTreeSet::new();
    let mut pending = vec![(directory.to_path_buf(), PathBuf::new())];
    while let Some((absolute, relative)) = pending.pop() {
        let entries = match fs::read_dir(&absolute) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error.into()),
        };
        for entry in entries {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let child_relative = relative.join(entry.file_name());
            if file_type.is_dir() {
                pending.push((entry.path(), child_relative));
            } else if entry.path().extension().and_then(|value| value.to_str()) == Some("ts") {
                files.insert(child_relative);
            }
        }
    }
    Ok(files)
}

fn write_if_changed(path: &Path, contents: &str) -> Result<(), CodegenError> {
    match fs::read_to_string(path) {
        Ok(current) if current == contents => Ok(()),
        Ok(_) => fs::write(path, contents).map_err(CodegenError::from),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let parent = path.parent().ok_or_else(|| {
                CodegenError::new(format!("generated path has no parent: {}", path.display()))
            })?;
            fs::create_dir_all(parent)?;
            fs::write(path, contents).map_err(CodegenError::from)
        }
        Err(error) => Err(error.into()),
    }
}
