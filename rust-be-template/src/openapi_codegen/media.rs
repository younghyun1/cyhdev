use serde_json::{Map, Value};

use super::{
    error::CodegenError,
    model::{RequestBody, SuccessResponse},
};

/// Parses the preferred request representation for a generated operation.
pub fn parse_request_body(
    value: Option<&Value>,
    method: &str,
    path: &str,
) -> Result<Option<RequestBody>, CodegenError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let request = object(value, &format!("request body for {method} {path}"))?;
    let content = request
        .get("content")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            CodegenError::new(format!("request body for {method} {path} has no content"))
        })?;
    let (content_type, media) = preferred_content(content).ok_or_else(|| {
        CodegenError::new(format!(
            "request body for {method} {path} has no supported content"
        ))
    })?;

    Ok(Some(RequestBody {
        content_type: content_type.to_owned(),
        required: request
            .get("required")
            .and_then(Value::as_bool)
            .unwrap_or(content_type == "multipart/form-data"),
        schema: media.get("schema").cloned(),
    }))
}

/// Parses the lowest successful response code and preferred representation.
pub fn parse_success_response(
    value: Option<&Value>,
    method: &str,
    path: &str,
) -> Result<SuccessResponse, CodegenError> {
    let responses = value
        .and_then(Value::as_object)
        .ok_or_else(|| CodegenError::new(format!("{method} {path} has no responses")))?;
    let (_, response) = responses
        .iter()
        .filter(|(status, _)| status.starts_with('2'))
        .min_by(|(left, _), (right, _)| left.cmp(right))
        .ok_or_else(|| CodegenError::new(format!("{method} {path} has no successful response")))?;
    let response = object(response, &format!("successful response for {method} {path}"))?;
    let Some(content) = response.get("content").and_then(Value::as_object) else {
        return Ok(SuccessResponse {
            content_type: None,
            schema: None,
        });
    };
    let Some((content_type, media)) = preferred_content(content) else {
        return Ok(SuccessResponse {
            content_type: None,
            schema: None,
        });
    };

    Ok(SuccessResponse {
        content_type: Some(content_type.to_owned()),
        schema: media.get("schema").cloned(),
    })
}

fn preferred_content(content: &Map<String, Value>) -> Option<(&str, &Map<String, Value>)> {
    for content_type in [
        "application/json",
        "multipart/form-data",
        "text/html",
        "text/plain",
    ] {
        if let Some(media) = content.get(content_type).and_then(Value::as_object) {
            return Some((content_type, media));
        }
    }
    content.iter().next().and_then(|(content_type, media)| {
        media
            .as_object()
            .map(|media| (content_type.as_str(), media))
    })
}

fn object<'a>(
    value: &'a Value,
    context: &str,
) -> Result<&'a Map<String, Value>, CodegenError> {
    value
        .as_object()
        .ok_or_else(|| CodegenError::new(format!("{context} is not an object")))
}
