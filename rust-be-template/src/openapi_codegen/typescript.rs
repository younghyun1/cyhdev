use std::collections::BTreeSet;

use serde_json::Value;

use super::{
    error::CodegenError,
    model::Contract,
    naming::{property_name, type_name},
};

pub const GENERATED_HEADER: &str =
    "// Generated from rust-be-template OpenAPI. Do not edit by hand.\n\n";

/// Renders the component schemas used by the selected operations.
pub fn render_types(contract: &Contract) -> Result<String, CodegenError> {
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

    for (name, schema) in &contract.components {
        output.push('\n');
        output.push_str("export type ");
        output.push_str(&type_name(name));
        output.push_str(" = ");
        output.push_str(&render_schema(schema, 0)?);
        output.push_str(";\n");
    }

    Ok(output)
}

/// Renders an OpenAPI schema as a strict TypeScript type expression.
pub fn render_schema(schema: &Value, indent: usize) -> Result<String, CodegenError> {
    let Some(schema) = schema.as_object() else {
        return Ok("unknown".to_owned());
    };

    let mut rendered = if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        reference
            .strip_prefix("#/components/schemas/")
            .map(type_name)
            .ok_or_else(|| CodegenError::new(format!("unsupported schema reference {reference}")))?
    } else if let Some(values) = schema.get("enum").and_then(Value::as_array) {
        render_enum(values)?
    } else if let Some(values) = schema.get("oneOf").and_then(Value::as_array) {
        render_composition(values, " | ", indent)?
    } else if let Some(values) = schema.get("anyOf").and_then(Value::as_array) {
        render_composition(values, " | ", indent)?
    } else if let Some(values) = schema.get("allOf").and_then(Value::as_array) {
        render_composition(values, " & ", indent)?
    } else {
        render_by_type(schema, indent)?
    };

    if schema.get("nullable").and_then(Value::as_bool) == Some(true)
        && !rendered.split(" | ").any(|item| item == "null")
    {
        rendered.push_str(" | null");
    }

    Ok(rendered)
}

/// Returns the inner data schema when a response uses the standard envelope.
pub fn enveloped_data_schema(schema: &Value) -> Option<&Value> {
    let properties = schema.as_object()?.get("properties")?.as_object()?;
    if properties.contains_key("success") && properties.contains_key("meta") {
        properties.get("data")
    } else {
        None
    }
}

/// Returns every named component referenced from a schema.
pub fn referenced_type_names(schema: &Value) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    collect_type_names(schema, &mut names);
    names
}

fn render_by_type(
    schema: &serde_json::Map<String, Value>,
    indent: usize,
) -> Result<String, CodegenError> {
    if let Some(types) = schema.get("type").and_then(Value::as_array) {
        let mut rendered = Vec::new();
        for value in types {
            let value = value
                .as_str()
                .ok_or_else(|| CodegenError::new("schema type arrays may only contain strings"))?;
            let value = render_primitive(value, schema, indent)?;
            if !rendered.contains(&value) {
                rendered.push(value);
            }
        }
        return Ok(rendered.join(" | "));
    }

    match schema.get("type").and_then(Value::as_str) {
        Some(schema_type) => render_primitive(schema_type, schema, indent),
        None if schema.contains_key("properties") => render_object(schema, indent),
        None => Ok("unknown".to_owned()),
    }
}

fn render_primitive(
    schema_type: &str,
    schema: &serde_json::Map<String, Value>,
    indent: usize,
) -> Result<String, CodegenError> {
    match schema_type {
        "array" => {
            if let Some(items) = schema.get("prefixItems").and_then(Value::as_array) {
                let items = items
                    .iter()
                    .map(|item| render_schema(item, indent))
                    .collect::<Result<Vec<_>, _>>()?;
                return Ok(format!("readonly [{}]", items.join(", ")));
            }
            let item = match schema.get("items") {
                Some(item) => render_schema(item, indent)?,
                None => "unknown".to_owned(),
            };
            Ok(format!("ReadonlyArray<{item}>"))
        }
        "boolean" => Ok("boolean".to_owned()),
        "integer" | "number" => Ok("number".to_owned()),
        "null" => Ok("null".to_owned()),
        "object" => render_object(schema, indent),
        "string" => Ok("string".to_owned()),
        unsupported => Err(CodegenError::new(format!(
            "unsupported OpenAPI schema type {unsupported}"
        ))),
    }
}

fn render_object(
    schema: &serde_json::Map<String, Value>,
    indent: usize,
) -> Result<String, CodegenError> {
    let properties = schema.get("properties").and_then(Value::as_object);
    let Some(properties) = properties else {
        return render_additional_properties(schema, indent);
    };
    if properties.is_empty() {
        return render_additional_properties(schema, indent);
    }

    let required: BTreeSet<&str> = schema
        .get("required")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .collect();
    let mut names: Vec<&String> = properties.keys().collect();
    names.sort();
    let child_indent = indent + 2;
    let mut output = String::from("{\n");
    for name in names {
        output.push_str(&" ".repeat(child_indent));
        output.push_str("readonly ");
        output.push_str(&property_name(name)?);
        if !required.contains(name.as_str()) {
            output.push('?');
        }
        output.push_str(": ");
        output.push_str(&render_schema(&properties[name], child_indent)?);
        output.push_str(";\n");
    }
    output.push_str(&" ".repeat(indent));
    output.push('}');
    Ok(output)
}

fn render_additional_properties(
    schema: &serde_json::Map<String, Value>,
    indent: usize,
) -> Result<String, CodegenError> {
    match schema.get("additionalProperties") {
        Some(Value::Bool(true)) => Ok("Readonly<Record<string, unknown>>".to_owned()),
        Some(Value::Object(_)) => Ok(format!(
            "Readonly<Record<string, {}>>",
            render_schema(&schema["additionalProperties"], indent)?
        )),
        _ => Ok("Readonly<Record<string, never>>".to_owned()),
    }
}

fn render_enum(values: &[Value]) -> Result<String, CodegenError> {
    if values.is_empty() {
        return Ok("never".to_owned());
    }
    values
        .iter()
        .map(serde_json::to_string)
        .collect::<Result<Vec<_>, _>>()
        .map(|values| values.join(" | "))
        .map_err(CodegenError::from)
}

fn render_composition(
    values: &[Value],
    separator: &str,
    indent: usize,
) -> Result<String, CodegenError> {
    if values.is_empty() {
        return Ok("unknown".to_owned());
    }
    let mut rendered = values
        .iter()
        .map(|value| render_schema(value, indent))
        .collect::<Result<Vec<_>, _>>()?;
    if separator == " | " {
        rendered.sort_by_key(|value| value == "null");
    }
    Ok(rendered.join(separator))
}

fn collect_type_names(value: &Value, names: &mut BTreeSet<String>) {
    match value {
        Value::Array(values) => {
            for value in values {
                collect_type_names(value, names);
            }
        }
        Value::Object(values) => {
            if let Some(reference) = values.get("$ref").and_then(Value::as_str)
                && let Some(name) = reference.strip_prefix("#/components/schemas/")
                && name != "ApiResponseMeta"
            {
                names.insert(type_name(name));
            }
            for value in values.values() {
                collect_type_names(value, names);
            }
        }
        _ => {}
    }
}
