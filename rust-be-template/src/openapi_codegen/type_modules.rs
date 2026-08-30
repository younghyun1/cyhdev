use std::collections::BTreeMap;

use serde_json::Value;

use super::{
    error::CodegenError,
    naming::{type_file_name, type_name},
    typescript::{GENERATED_HEADER, referenced_type_names, render_schema},
};

/// Renders the shared JSON response envelope.
pub fn render_api_response() -> String {
    let mut output = String::from(GENERATED_HEADER);
    output.push_str(
        "export type ApiResponseMeta = {\n\
         \x20 readonly time_to_process: string;\n\
         \x20 readonly timestamp: string;\n\
         \x20 readonly metadata: unknown;\n\
         };\n\n\
         export type ApiResponse<T> = {\n\
         \x20 readonly success: boolean;\n\
         \x20 readonly data: T;\n\
         \x20 readonly meta: ApiResponseMeta;\n\
         };\n",
    );
    output
}

/// Renders one component schema with direct type-only dependency imports.
pub fn render_component(name: &str, schema: &Value) -> Result<String, CodegenError> {
    let rendered_name = type_name(name);
    let mut dependencies = referenced_type_names(schema);
    dependencies.remove(&rendered_name);
    let mut output = String::from(GENERATED_HEADER);
    for dependency in dependencies {
        output.push_str("import type { ");
        output.push_str(&dependency);
        output.push_str(" } from \"./");
        output.push_str(&type_file_name(&dependency));
        output.push_str("\";\n");
    }
    if output.lines().count() > 1 {
        output.push('\n');
    }
    output.push_str("export type ");
    output.push_str(&rendered_name);
    output.push_str(" = ");
    output.push_str(&render_schema(schema, 0)?);
    output.push_str(";\n");
    Ok(output)
}

/// Renders the stable type barrel in component-name order.
pub fn render_type_index(components: &BTreeMap<String, Value>) -> String {
    let mut output = String::from(GENERATED_HEADER);
    output.push_str("export * from \"./api-response\";\n");
    for name in components.keys() {
        output.push_str("export * from \"./");
        output.push_str(&type_file_name(name));
        output.push_str("\";\n");
    }
    output
}
