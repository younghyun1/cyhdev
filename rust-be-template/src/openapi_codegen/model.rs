use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value};

use super::{
    error::CodegenError,
    media::{parse_request_body, parse_success_response},
    reference::{collect_components, collect_schema_refs},
    surface::FrontendOperation,
};

/// HTTP parameter position represented by OpenAPI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParameterLocation {
    Path,
    Query,
}

/// Parameter needed to invoke one generated operation.
#[derive(Clone, Debug)]
pub struct Parameter {
    pub location: ParameterLocation,
    pub name: String,
    pub required: bool,
    pub schema: Value,
}

/// Request body accepted by one generated operation.
#[derive(Clone, Debug)]
pub struct RequestBody {
    pub content_type: String,
    pub required: bool,
    pub schema: Option<Value>,
}

/// Successful response returned by one generated operation.
#[derive(Clone, Debug)]
pub struct SuccessResponse {
    pub content_type: Option<String>,
    pub schema: Option<Value>,
}

/// OpenAPI operation selected for the generated client.
#[derive(Clone, Debug)]
pub struct Operation {
    pub client_group: String,
    pub method: String,
    pub operation_id: String,
    pub parameters: Vec<Parameter>,
    pub path: String,
    pub request_body: Option<RequestBody>,
    pub response: SuccessResponse,
}

/// Selected operations and the transitive component schemas they use.
#[derive(Debug)]
pub struct Contract {
    pub components: BTreeMap<String, Value>,
    pub operations: Vec<Operation>,
}

/// Selects the explicit frontend operations and their reachable component schemas.
pub fn select_contract(
    spec: &Value,
    selected_operations: &[FrontendOperation],
) -> Result<Contract, CodegenError> {
    let root = object(spec, "OpenAPI document")?;
    let paths = root
        .get("paths")
        .ok_or_else(|| CodegenError::new("OpenAPI document has no paths"))?;
    let path_map = object(paths, "OpenAPI paths")?;
    let selected = selected_operation_map(selected_operations)?;
    let mut operations = Vec::new();
    let mut found = BTreeSet::new();
    let mut referenced_schemas = BTreeSet::new();

    for (path, path_item) in path_map {
        let path_item = object(path_item, &format!("path item {path}"))?;
        for method in ["delete", "get", "head", "options", "patch", "post", "put", "trace"] {
            let Some(operation) = path_item.get(method) else {
                continue;
            };
            let operation_map = object(operation, &format!("{method} {path}"))?;
            let method = method.to_ascii_uppercase();
            let Some(client_group) = selected.get(&(method.as_str(), path.as_str())) else {
                continue;
            };

            let parsed = parse_operation(&method, path, operation_map, client_group)?;
            collect_operation_schema_refs(&parsed, &mut referenced_schemas);
            found.insert((method, path.to_owned()));
            operations.push(parsed);
        }
    }

    validate_selected_operations(&selected, &found)?;
    operations.sort_by(|left, right| {
        left.client_group
            .cmp(&right.client_group)
            .then_with(|| left.operation_id.cmp(&right.operation_id))
    });
    let schemas = root
        .get("components")
        .and_then(Value::as_object)
        .and_then(|components| components.get("schemas"))
        .and_then(Value::as_object)
        .ok_or_else(|| CodegenError::new("OpenAPI document has no component schemas"))?;
    let components = collect_components(schemas, referenced_schemas)?;

    Ok(Contract {
        components,
        operations,
    })
}

fn collect_operation_schema_refs(operation: &Operation, names: &mut BTreeSet<String>) {
    for parameter in &operation.parameters {
        collect_schema_refs(&parameter.schema, names);
    }
    if let Some(body) = &operation.request_body
        && let Some(schema) = &body.schema
    {
        collect_schema_refs(schema, names);
    }
    if let Some(schema) = &operation.response.schema {
        collect_schema_refs(schema, names);
    }
}

fn parse_operation(
    method: &str,
    path: &str,
    operation: &Map<String, Value>,
    client_group: &str,
) -> Result<Operation, CodegenError> {
    let operation_id = operation
        .get("operationId")
        .and_then(Value::as_str)
        .ok_or_else(|| CodegenError::new(format!("{method} {path} has no operationId")))?;
    let parameters = parse_parameters(operation.get("parameters"), method, path)?;
    let request_body = parse_request_body(operation.get("requestBody"), method, path)?;
    let response = parse_success_response(operation.get("responses"), method, path)?;

    Ok(Operation {
        client_group: client_group.to_owned(),
        method: method.to_owned(),
        operation_id: operation_id.to_owned(),
        parameters,
        path: path.to_owned(),
        request_body,
        response,
    })
}

fn parse_parameters(
    value: Option<&Value>,
    method: &str,
    path: &str,
) -> Result<Vec<Parameter>, CodegenError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let values = value.as_array().ok_or_else(|| {
        CodegenError::new(format!("parameters for {method} {path} are not an array"))
    })?;
    let mut parameters = Vec::new();

    for value in values {
        let parameter = object(value, &format!("parameter for {method} {path}"))?;
        let name = required_string(parameter, "name", &format!("parameter for {method} {path}"))?;
        let location = match required_string(
            parameter,
            "in",
            &format!("parameter {name} for {method} {path}"),
        )?
        .as_str()
        {
            "path" => ParameterLocation::Path,
            "query" => ParameterLocation::Query,
            unsupported => {
                return Err(CodegenError::new(format!(
                    "unsupported {unsupported} parameter {name} for {method} {path}"
                )));
            }
        };
        let schema = parameter.get("schema").cloned().ok_or_else(|| {
            CodegenError::new(format!("parameter {name} for {method} {path} has no schema"))
        })?;
        parameters.push(Parameter {
            location,
            name,
            required: parameter
                .get("required")
                .and_then(Value::as_bool)
                .unwrap_or(matches!(location, ParameterLocation::Path)),
            schema,
        });
    }

    parameters.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(parameters)
}

fn selected_operation_map(
    operations: &[FrontendOperation],
) -> Result<BTreeMap<(&str, &str), &str>, CodegenError> {
    let mut selected = BTreeMap::new();
    for operation in operations {
        let key = (operation.method, operation.path);
        if selected.insert(key, operation.client_group).is_some() {
            return Err(CodegenError::new(format!(
                "frontend contract repeats {} {}",
                operation.method, operation.path
            )));
        }
    }
    Ok(selected)
}

fn validate_selected_operations(
    selected: &BTreeMap<(&str, &str), &str>,
    found: &BTreeSet<(String, String)>,
) -> Result<(), CodegenError> {
    let missing: Vec<String> = selected
        .keys()
        .filter(|(method, path)| !found.contains(&(method.to_string(), path.to_string())))
        .map(|(method, path)| format!("{method} {path}"))
        .collect();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(CodegenError::new(format!(
            "OpenAPI is missing frontend operations: {}",
            missing.join(", ")
        )))
    }
}

fn object<'a>(value: &'a Value, context: &str) -> Result<&'a Map<String, Value>, CodegenError> {
    value
        .as_object()
        .ok_or_else(|| CodegenError::new(format!("{context} is not an object")))
}

fn required_string(
    value: &Map<String, Value>,
    field: &str,
    context: &str,
) -> Result<String, CodegenError> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| CodegenError::new(format!("{context} has no {field}")))
}
