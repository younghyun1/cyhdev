use std::{error::Error, fmt};

/// Failure while reading or rendering the OpenAPI contract.
#[derive(Debug)]
pub struct CodegenError {
    message: String,
}

impl CodegenError {
    /// Creates a code-generation error with actionable context.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for CodegenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for CodegenError {}

impl From<std::io::Error> for CodegenError {
    fn from(error: std::io::Error) -> Self {
        Self::new(format!("filesystem operation failed: {error}"))
    }
}

impl From<serde_json::Error> for CodegenError {
    fn from(error: serde_json::Error) -> Self {
        Self::new(format!("OpenAPI serialization failed: {error}"))
    }
}
