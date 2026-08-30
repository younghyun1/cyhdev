use super::error::CodegenError;

/// Converts an OpenAPI operation identifier to the generated method name.
pub fn method_name(operation_id: &str) -> String {
    let operation_id = operation_id
        .strip_suffix("_handler")
        .or_else(|| operation_id.strip_suffix("_process"))
        .unwrap_or(operation_id);
    snake_to_camel(operation_id)
}

/// Produces a valid TypeScript property key.
pub fn property_name(name: &str) -> Result<String, CodegenError> {
    if is_identifier(name) {
        Ok(name.to_owned())
    } else {
        serde_json::to_string(name).map_err(CodegenError::from)
    }
}

/// Produces a stable TypeScript identifier from a component name.
pub fn type_name(name: &str) -> String {
    name.chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect()
}

/// Produces the stable kebab-case filename for one generated TypeScript type.
pub fn type_file_name(name: &str) -> String {
    let mut output = String::new();
    let mut previous_was_lowercase_or_digit = false;
    for character in name.chars() {
        if character.is_ascii_uppercase() {
            if previous_was_lowercase_or_digit {
                output.push('-');
            }
            output.push(character.to_ascii_lowercase());
            previous_was_lowercase_or_digit = false;
        } else if character.is_ascii_alphanumeric() {
            output.push(character.to_ascii_lowercase());
            previous_was_lowercase_or_digit = true;
        } else if !output.ends_with('-') {
            output.push('-');
            previous_was_lowercase_or_digit = false;
        }
    }
    output.trim_matches('-').to_owned()
}

/// Produces a client factory name from a kebab-case group name.
pub fn client_factory_name(group: &str) -> String {
    let mut output = String::from("create");
    for part in group.split('-').filter(|part| !part.is_empty()) {
        let mut characters = part.chars();
        if let Some(first) = characters.next() {
            output.extend(first.to_uppercase());
            output.extend(characters);
        }
    }
    output.push_str("Client");
    output
}

fn snake_to_camel(value: &str) -> String {
    let mut output = String::new();
    let mut uppercase = false;
    for character in value.chars() {
        if character == '_' {
            uppercase = true;
        } else if uppercase {
            output.extend(character.to_uppercase());
            uppercase = false;
        } else {
            output.push(character);
        }
    }
    output
}

fn is_identifier(value: &str) -> bool {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_' || first == '$')
        && characters.all(|character| {
            character.is_ascii_alphanumeric() || character == '_' || character == '$'
        })
}
