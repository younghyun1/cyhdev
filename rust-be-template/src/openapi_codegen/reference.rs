use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde_json::{Map, Value};

use super::error::CodegenError;

/// Resolves a transitive component-schema closure in stable name order.
pub fn collect_components(
    schemas: &Map<String, Value>,
    initial: BTreeSet<String>,
) -> Result<BTreeMap<String, Value>, CodegenError> {
    let mut pending: VecDeque<String> = initial.into_iter().collect();
    let mut components = BTreeMap::new();

    while let Some(name) = pending.pop_front() {
        if components.contains_key(&name) || name == "ApiResponseMeta" {
            continue;
        }
        let schema = schemas.get(&name).cloned().ok_or_else(|| {
            CodegenError::new(format!("operation references missing component schema {name}"))
        })?;
        let mut dependencies = BTreeSet::new();
        collect_schema_refs(&schema, &mut dependencies);
        pending.extend(dependencies);
        components.insert(name, schema);
    }

    Ok(components)
}

/// Collects local OpenAPI component-schema references from a JSON value.
pub fn collect_schema_refs(value: &Value, names: &mut BTreeSet<String>) {
    match value {
        Value::Array(values) => {
            for value in values {
                collect_schema_refs(value, names);
            }
        }
        Value::Object(values) => {
            if let Some(reference) = values.get("$ref").and_then(Value::as_str)
                && let Some(name) = reference.strip_prefix("#/components/schemas/")
            {
                names.insert(name.to_owned());
            }
            for value in values.values() {
                collect_schema_refs(value, names);
            }
        }
        _ => {}
    }
}
