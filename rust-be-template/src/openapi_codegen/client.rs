use std::collections::BTreeSet;

use super::{
    error::CodegenError,
    model::{Operation, ParameterLocation, RequestBody},
    naming::{method_name, property_name},
    typescript::{GENERATED_HEADER, enveloped_data_schema, referenced_type_names, render_schema},
};

/// Renders one transport-injected client group.
pub fn render_client(operations: &[Operation], factory_name: &str) -> Result<String, CodegenError> {
    validate_method_names(operations)?;
    let imports = collect_imports(operations);
    let mut output = String::from(GENERATED_HEADER);
    output.push_str("import type {\n");
    for import in imports {
        output.push_str("  ");
        output.push_str(&import);
        output.push_str(",\n");
    }
    output.push_str("} from \"../api-types\";\n");
    output.push_str("import {\n");
    for import in collect_runtime_imports(operations) {
        output.push_str("  ");
        output.push_str(import);
        output.push_str(",\n");
    }
    output.push_str(
        "  type ApiRequestOptions,\n\
         \x20 type ApiTransport,\n\
         } from \"../runtime\";\n\n",
    );
    output.push_str("export function ");
    output.push_str(factory_name);
    output.push_str("(transport: ApiTransport) {\n  return {\n");

    for operation in operations {
        render_operation(&mut output, operation)?;
    }

    output.push_str("  } as const;\n}\n");
    Ok(output)
}

fn render_operation(output: &mut String, operation: &Operation) -> Result<(), CodegenError> {
    let has_input = !operation.parameters.is_empty() || operation.request_body.is_some();
    let input_required = operation
        .parameters
        .iter()
        .any(|parameter| parameter.required)
        || operation
            .request_body
            .as_ref()
            .is_some_and(|body| body.required);
    let method = method_name(&operation.operation_id);
    output.push_str("    ");
    output.push_str(&method);
    output.push_str(": async (");
    if has_input {
        output.push_str("input: ");
        output.push_str(&render_input_type(operation)?);
        if !input_required {
            output.push_str(" = {}");
        }
        output.push_str(", ");
    }
    output.push_str("options: ApiRequestOptions = {}) => {\n");

    let has_path = operation
        .parameters
        .iter()
        .any(|parameter| parameter.location == ParameterLocation::Path);
    let has_query = operation
        .parameters
        .iter()
        .any(|parameter| parameter.location == ParameterLocation::Query);
    if has_path {
        output.push_str("      const path = interpolatePath(\"");
        output.push_str(&operation.path);
        output.push_str("\", input.path);\n");
    } else {
        output.push_str("      const path = \"");
        output.push_str(&operation.path);
        output.push_str("\";\n");
    }
    if has_query {
        output.push_str("      const url = appendQuery(path, input.query);\n");
    } else {
        output.push_str("      const url = path;\n");
    }

    let response_type = render_response_type(operation)?;
    let request_function = match operation.response.content_type.as_deref() {
        Some("application/json") => "requestJson",
        Some("text/html" | "text/plain") => "requestText",
        Some(content_type) => {
            return Err(CodegenError::new(format!(
                "unsupported success content type {content_type} for {} {}",
                operation.method, operation.path
            )));
        }
        None => {
            return Err(CodegenError::new(format!(
                "{} {} has no documented success content",
                operation.method, operation.path
            )));
        }
    };
    output.push_str("      return ");
    output.push_str(request_function);
    output.push('<');
    output.push_str(&response_type);
    output.push_str(">(transport, url, {\n        method: \"");
    output.push_str(&operation.method);
    output.push_str("\",\n        headers: requestHeaders(options.headers, ");
    output.push_str(if is_json_body(operation.request_body.as_ref()) {
        "true"
    } else {
        "false"
    });
    output.push_str("),\n        signal: options.signal,\n");
    if let Some(body) = &operation.request_body {
        output.push_str("        body: ");
        if body.content_type == "application/json" {
            output.push_str("JSON.stringify(input.body)");
        } else {
            output.push_str("input.body");
        }
        output.push_str(",\n");
    }
    output.push_str("      });\n    },\n");
    Ok(())
}

fn render_input_type(operation: &Operation) -> Result<String, CodegenError> {
    let mut output = String::from("{\n");
    if let Some(body) = &operation.request_body {
        output.push_str("      readonly body");
        if !body.required {
            output.push('?');
        }
        output.push_str(": ");
        output.push_str(&render_body_type(body)?);
        output.push_str(";\n");
    }
    for (location, property) in [
        (ParameterLocation::Path, "path"),
        (ParameterLocation::Query, "query"),
    ] {
        let parameters: Vec<_> = operation
            .parameters
            .iter()
            .filter(|parameter| parameter.location == location)
            .collect();
        if parameters.is_empty() {
            continue;
        }
        output.push_str("      readonly ");
        output.push_str(property);
        if !parameters.iter().any(|parameter| parameter.required) {
            output.push('?');
        }
        output.push_str(": {\n");
        for parameter in parameters {
            output.push_str("        readonly ");
            output.push_str(&property_name(&parameter.name)?);
            if !parameter.required {
                output.push('?');
            }
            output.push_str(": ");
            output.push_str(&render_schema(&parameter.schema, 8)?);
            output.push_str(";\n");
        }
        output.push_str("      };\n");
    }
    output.push_str("    }");
    Ok(output)
}

fn render_body_type(body: &RequestBody) -> Result<String, CodegenError> {
    if body.content_type == "multipart/form-data" {
        Ok("FormData".to_owned())
    } else {
        match &body.schema {
            Some(schema) => render_schema(schema, 6),
            None => Ok("unknown".to_owned()),
        }
    }
}

fn render_response_type(operation: &Operation) -> Result<String, CodegenError> {
    match operation.response.content_type.as_deref() {
        Some("application/json") => match &operation.response.schema {
            Some(schema) => match enveloped_data_schema(schema) {
                Some(data) => Ok(format!("ApiResponse<{}>", render_schema(data, 0)?)),
                None => render_schema(schema, 0),
            },
            None => Ok("unknown".to_owned()),
        },
        Some("text/html" | "text/plain") => Ok("string".to_owned()),
        Some(content_type) => Err(CodegenError::new(format!(
            "unsupported success content type {content_type} for {} {}",
            operation.method, operation.path
        ))),
        None => Err(CodegenError::new(format!(
            "{} {} has no documented success content",
            operation.method, operation.path
        ))),
    }
}

fn collect_imports(operations: &[Operation]) -> BTreeSet<String> {
    let mut imports = BTreeSet::new();
    for operation in operations {
        for parameter in &operation.parameters {
            imports.extend(referenced_type_names(&parameter.schema));
        }
        if let Some(body) = &operation.request_body
            && let Some(schema) = &body.schema
        {
            imports.extend(referenced_type_names(schema));
        }
        if let Some(schema) = &operation.response.schema {
            imports.extend(referenced_type_names(schema));
            if enveloped_data_schema(schema).is_some() {
                imports.insert("ApiResponse".to_owned());
            }
        }
    }
    imports
}

fn collect_runtime_imports(operations: &[Operation]) -> BTreeSet<&'static str> {
    let mut imports = BTreeSet::from(["requestHeaders"]);
    for operation in operations {
        if operation
            .parameters
            .iter()
            .any(|parameter| parameter.location == ParameterLocation::Path)
        {
            imports.insert("interpolatePath");
        }
        if operation
            .parameters
            .iter()
            .any(|parameter| parameter.location == ParameterLocation::Query)
        {
            imports.insert("appendQuery");
        }
        match operation.response.content_type.as_deref() {
            Some("application/json") => {
                imports.insert("requestJson");
            }
            Some("text/html" | "text/plain") => {
                imports.insert("requestText");
            }
            _ => {}
        }
    }
    imports
}

/// Rejects operation identifiers that collapse to the same TypeScript method name.
pub fn validate_method_names(operations: &[Operation]) -> Result<(), CodegenError> {
    let mut names = BTreeSet::new();
    for operation in operations {
        let name = method_name(&operation.operation_id);
        if !names.insert(name.clone()) {
            return Err(CodegenError::new(format!(
                "multiple selected operations generate the method name {name}"
            )));
        }
    }
    Ok(())
}

fn is_json_body(body: Option<&RequestBody>) -> bool {
    body.is_some_and(|body| body.content_type == "application/json")
}
