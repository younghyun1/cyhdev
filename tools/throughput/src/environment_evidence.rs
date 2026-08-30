//! Required exact input evidence for hardware-specific HTTP comparisons.

use std::path::Path;

use crate::{
    config::EnvironmentConfig,
    error::{HarnessError, HarnessResult},
};

const DIGEST_KEYS: [&str; 5] = [
    "working_tree_digest",
    "database_schema_digest",
    "database_dataset_digest",
    "geo_ipv4_digest",
    "geo_ipv6_digest",
];
const EXACT_TEXT_KEYS: [&str; 3] = [
    "database_server_version",
    "openssl_version",
    "socat_version",
];

pub fn validate(path: &Path, environment: &EnvironmentConfig) -> HarnessResult<()> {
    for key in DIGEST_KEYS {
        match environment.configuration.get(key) {
            Some(value) if is_sha256(value) => {}
            Some(_) | None => {
                return Err(HarnessError::Configuration {
                    path: path.to_path_buf(),
                    detail: format!("HTTP environment `{key}` must be an exact sha256 digest"),
                });
            }
        }
    }
    for key in EXACT_TEXT_KEYS {
        match environment.configuration.get(key) {
            Some(value)
                if !value.to_ascii_lowercase().contains("replace") && !value.contains(".x") => {}
            Some(_) | None => {
                return Err(HarnessError::Configuration {
                    path: path.to_path_buf(),
                    detail: format!("HTTP environment must declare exact `{key}` evidence"),
                });
            }
        }
    }
    Ok(())
}

fn is_sha256(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, path::Path};

    use crate::config::EnvironmentConfig;

    #[test]
    fn rejects_placeholder_http_evidence() {
        let environment = EnvironmentConfig {
            schema_version: 1,
            label: "http".to_owned(),
            power_profile: "performance".to_owned(),
            build_profile: "debug".to_owned(),
            configuration: BTreeMap::from([(
                "database_server_version".to_owned(),
                "PostgreSQL 18.x".to_owned(),
            )]),
            notes: Vec::new(),
        };
        assert!(super::validate(Path::new("<test>"), &environment).is_err());
    }
}
