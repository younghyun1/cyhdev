//! Source-derived Axum route inventory checked against the OpenAPI document.

use std::collections::BTreeSet;

use serde_json::Value;

use super::error::CodegenError;

const ROUTER_SOURCE: &str = include_str!("../routers/main_router.rs");
const HTTP_METHODS: [&str; 8] = [
    "delete", "get", "head", "options", "patch", "post", "put", "trace",
];

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Operation {
    method: String,
    path: String,
}

/// Rejects any method/path present on only one side of router registration and OpenAPI.
pub fn validate(spec: &Value) -> Result<(), CodegenError> {
    let router = parse_router_operations(ROUTER_SOURCE)?;
    let openapi = openapi_operations(spec)?;
    let undocumented = router.difference(&openapi).cloned().collect::<Vec<_>>();
    let unregistered = openapi.difference(&router).cloned().collect::<Vec<_>>();
    if undocumented.is_empty() && unregistered.is_empty() {
        return Ok(());
    }
    Err(CodegenError::new(format!(
        "router/OpenAPI operation drift; registered but undocumented: {}; documented but unregistered: {}",
        display_operations(&undocumented),
        display_operations(&unregistered),
    )))
}

fn parse_router_operations(source: &str) -> Result<BTreeSet<Operation>, CodegenError> {
    let mut operations = BTreeSet::new();
    let bytes = source.as_bytes();
    let mut cursor = 0usize;
    while let Some(relative) = source[cursor..].find(".route") {
        let route_start = cursor.saturating_add(relative);
        let mut index = route_start.saturating_add(".route".len());
        skip_ascii_whitespace(bytes, &mut index);
        if bytes.get(index) != Some(&b'(') {
            cursor = index;
            continue;
        }
        index = index.saturating_add(1);
        skip_ascii_whitespace(bytes, &mut index);
        let (path, after_path) = parse_string_literal(source, index)?;
        index = after_path;
        skip_ascii_whitespace(bytes, &mut index);
        if bytes.get(index) != Some(&b',') {
            return Err(CodegenError::new(format!(
                "router route `{path}` has no method-router argument"
            )));
        }
        let expression_start = index.saturating_add(1);
        let expression_end = find_call_end(source, expression_start)?;
        if !path.starts_with("/ws/") {
            let expression = &source[expression_start..expression_end];
            let methods = methods_in_expression(expression);
            if methods.is_empty() {
                return Err(CodegenError::new(format!(
                    "router route `{path}` has no recognized HTTP method"
                )));
            }
            for method in methods {
                let operation = Operation {
                    method: method.to_ascii_uppercase(),
                    path: path.clone(),
                };
                if !operations.insert(operation.clone()) {
                    return Err(CodegenError::new(format!(
                        "router registers {} {} more than once",
                        operation.method, operation.path
                    )));
                }
            }
        }
        cursor = expression_end.saturating_add(1);
    }
    if operations.is_empty() {
        Err(CodegenError::new(
            "could not derive any HTTP routes from main_router.rs",
        ))
    } else {
        Ok(operations)
    }
}

fn parse_string_literal(source: &str, start: usize) -> Result<(String, usize), CodegenError> {
    let bytes = source.as_bytes();
    if bytes.get(start) != Some(&b'"') {
        return Err(CodegenError::new(
            "router paths must use ordinary string literals",
        ));
    }
    let mut index = start.saturating_add(1);
    let mut escaped = false;
    while let Some(byte) = bytes.get(index).copied() {
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == b'"' {
            let raw = &source[start..=index];
            let path = serde_json::from_str::<String>(raw)?;
            return Ok((path, index.saturating_add(1)));
        }
        index = index.saturating_add(1);
    }
    Err(CodegenError::new("unterminated router path literal"))
}

fn find_call_end(source: &str, start: usize) -> Result<usize, CodegenError> {
    let bytes = source.as_bytes();
    let mut depth = 1usize;
    let mut index = start;
    let mut in_string = false;
    let mut escaped = false;
    while let Some(byte) = bytes.get(index).copied() {
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
        } else {
            match byte {
                b'"' => in_string = true,
                b'(' => depth = depth.saturating_add(1),
                b')' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return Ok(index);
                    }
                }
                _ => {}
            }
        }
        index = index.saturating_add(1);
    }
    Err(CodegenError::new("unterminated router route call"))
}

fn methods_in_expression(expression: &str) -> BTreeSet<&'static str> {
    HTTP_METHODS
        .into_iter()
        .filter(|method| expression_contains_call(expression, method))
        .collect()
}

fn expression_contains_call(expression: &str, method: &str) -> bool {
    let needle = format!("{method}(");
    expression.match_indices(&needle).any(|(index, _)| {
        index == 0 || {
            let preceding = expression.as_bytes()[index.saturating_sub(1)];
            !preceding.is_ascii_alphanumeric() && preceding != b'_'
        }
    })
}

fn openapi_operations(spec: &Value) -> Result<BTreeSet<Operation>, CodegenError> {
    let paths = spec
        .get("paths")
        .and_then(Value::as_object)
        .ok_or_else(|| CodegenError::new("OpenAPI document has no path map"))?;
    let mut operations = BTreeSet::new();
    for (path, item) in paths {
        let methods = item
            .as_object()
            .ok_or_else(|| CodegenError::new(format!("OpenAPI path `{path}` is not an object")))?;
        for method in HTTP_METHODS {
            if methods.contains_key(method) {
                operations.insert(Operation {
                    method: method.to_ascii_uppercase(),
                    path: path.clone(),
                });
            }
        }
    }
    Ok(operations)
}

fn skip_ascii_whitespace(bytes: &[u8], index: &mut usize) {
    while bytes.get(*index).is_some_and(u8::is_ascii_whitespace) {
        *index = index.saturating_add(1);
    }
}

fn display_operations(operations: &[Operation]) -> String {
    if operations.is_empty() {
        return "none".to_owned();
    }
    operations
        .iter()
        .map(|operation| format!("{} {}", operation.method, operation.path))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::parse_router_operations;

    #[test]
    fn derives_chained_methods_and_ignores_websockets() -> Result<(), String> {
        let source = r#"
            Router::new()
                .route("/api/items", get(list).post(create))
                .route("/api/items/{item_id}", patch(update).delete(remove))
                .route("/ws/items", get(upgrade));
        "#;
        let operations = parse_router_operations(source).map_err(|error| error.to_string())?;
        let rendered = operations
            .into_iter()
            .map(|operation| format!("{} {}", operation.method, operation.path))
            .collect::<Vec<_>>();
        assert_eq!(
            rendered,
            [
                "DELETE /api/items/{item_id}",
                "GET /api/items",
                "PATCH /api/items/{item_id}",
                "POST /api/items",
            ]
        );
        Ok(())
    }
}
