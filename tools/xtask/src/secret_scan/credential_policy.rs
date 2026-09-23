//! Filename-only runtime credential classification; file contents are never read.

use std::{
    ffi::OsStr,
    path::{Component, Path},
};

pub(super) const RUNTIME_CREDENTIAL_PATHS: [&str; 15] = [
    ":(glob)**/.env",
    ":(glob)**/.env.*",
    ":(glob)**/*credentials*",
    ":(glob)**/service-account.json",
    ":(glob)**/certs/**",
    ":(glob)**/*.pem",
    ":(glob)**/*.key",
    ":(glob)**/*.p12",
    ":(glob)**/*.pfx",
    ":(glob)**/*.jks",
    ":(glob)**/*.keystore",
    ":(glob)**/*.bks",
    ":(glob)**/.pgpass",
    ":(glob)**/.netrc",
    ":(glob)**/db/password.txt",
];

/// Key and keystore formats. The binary ones (PKCS #12, JKS, BKS) are opaque
/// to gitleaks, so the file name is the only available signal.
const KEY_FILE_EXTENSIONS: [&str; 7] = ["pem", "key", "p12", "pfx", "jks", "keystore", "bks"];

/// Plaintext client password files read by libpq, curl, and FTP clients.
const PASSWORD_FILE_NAMES: [&str; 2] = [".pgpass", ".netrc"];

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
    let extension = Path::new(name).extension().and_then(OsStr::to_str);
    let credential_named = name.contains("credentials")
        && !matches!(extension, Some("rs" | "ts" | "tsx" | "js" | "jsx" | "md"));
    name == ".env"
        || name.starts_with(".env.")
        || name == "service-account.json"
        || credential_named
        || extension.is_some_and(|extension| KEY_FILE_EXTENSIONS.contains(&extension))
        || PASSWORD_FILE_NAMES.contains(&name.as_str())
        || components.iter().any(|component| component == "certs")
        || (name == "password.txt" && components.iter().any(|component| component == "db"))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{RUNTIME_CREDENTIAL_PATHS, is_runtime_credential_path};

    #[test]
    fn recognizes_private_runtime_credentials_without_reading_them() {
        assert!(is_runtime_credential_path(Path::new(
            "rust-be-template/.env"
        )));
        assert!(is_runtime_credential_path(Path::new(
            "rust-be-template/certs/key.pem"
        )));
        assert!(is_runtime_credential_path(Path::new("db/password.txt")));
        assert!(!is_runtime_credential_path(Path::new(
            "rust-be-template/.env.example"
        )));
        assert!(!is_runtime_credential_path(Path::new("src/credentials.rs")));
    }

    #[test]
    fn recognizes_binary_keystores_and_client_password_files() {
        for path in [
            "android/release.jks",
            "android/app/upload.KEYSTORE",
            "config/truststore.bks",
            ".pgpass",
            "tools/.pgpass",
            ".netrc",
            "home/fixture/.netrc",
        ] {
            assert!(is_runtime_credential_path(Path::new(path)), "{path}");
        }
        for path in [
            "docs/keystore.md",
            "src/pgpass.rs",
            "src/netrc_parser.rs",
            "fixtures/pgpass.example",
        ] {
            assert!(!is_runtime_credential_path(Path::new(path)), "{path}");
        }
    }

    #[test]
    fn git_pathspecs_list_every_new_credential_name() {
        for pattern in [
            ":(glob)**/*.jks",
            ":(glob)**/*.keystore",
            ":(glob)**/*.bks",
            ":(glob)**/.pgpass",
            ":(glob)**/.netrc",
        ] {
            assert!(RUNTIME_CREDENTIAL_PATHS.contains(&pattern), "{pattern}");
        }
    }
}
