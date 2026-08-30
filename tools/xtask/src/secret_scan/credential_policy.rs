//! Filename-only runtime credential classification; file contents are never read.

use std::{ffi::OsStr, path::{Component, Path}};

pub(super) const RUNTIME_CREDENTIAL_PATHS: [&str; 10] = [
    ":(glob)**/.env",
    ":(glob)**/.env.*",
    ":(glob)**/*credentials*",
    ":(glob)**/service-account.json",
    ":(glob)**/certs/**",
    ":(glob)**/*.pem",
    ":(glob)**/*.key",
    ":(glob)**/*.p12",
    ":(glob)**/*.pfx",
    ":(glob)**/db/password.txt",
];

pub(super) fn is_runtime_credential_path(path: &Path) -> bool {
    let components = path
        .components()
        .filter_map(|component| match component {
            Component::Normal(value) => Some(value.to_string_lossy().to_ascii_lowercase()),
            _ => None,
        })
        .collect::<Vec<_>>();
    let Some(name) = components.last() else {
        return false;
    };
    if name == ".env.example" || name.ends_with(".env.example") {
        return false;
    }
    name == ".env"
        || name.starts_with(".env.")
        || name == "service-account.json"
        || name.contains("credentials")
        || matches!(
            Path::new(name).extension().and_then(OsStr::to_str),
            Some("pem" | "key" | "p12" | "pfx")
        )
        || components.iter().any(|component| component == "certs")
        || (name == "password.txt" && components.iter().any(|component| component == "db"))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::is_runtime_credential_path;

    #[test]
    fn recognizes_private_runtime_credentials_without_reading_them() {
        assert!(is_runtime_credential_path(Path::new("rust-be-template/.env")));
        assert!(is_runtime_credential_path(Path::new("rust-be-template/certs/key.pem")));
        assert!(is_runtime_credential_path(Path::new("db/password.txt")));
        assert!(!is_runtime_credential_path(Path::new("rust-be-template/.env.example")));
        assert!(!is_runtime_credential_path(Path::new("src/credentials.rs")));
    }
}
