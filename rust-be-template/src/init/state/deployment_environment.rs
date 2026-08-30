#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DeploymentEnvironment {
    Local,
    Dev,
    Staging,
    Prod,
}

impl DeploymentEnvironment {
    /// Loads an explicit deployment mode; unknown or missing values abort startup.
    pub fn from_env() -> anyhow::Result<Self> {
        let configured = std::env::var("CURR_ENV")
            .map_err(|error| anyhow::anyhow!("CURR_ENV must be set: {error}"))?;
        Self::parse(&configured)
    }

    fn parse(configured: &str) -> anyhow::Result<Self> {
        match configured.trim().to_ascii_lowercase().as_str() {
            "local" | "localhost" => Ok(Self::Local),
            "dev" | "develop" | "development" => Ok(Self::Dev),
            "staging" | "stage" | "stg" => Ok(Self::Staging),
            "prd" | "prod" | "production" => Ok(Self::Prod),
            _ => Err(anyhow::anyhow!(
                "CURR_ENV has unsupported value {configured:?}"
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DeploymentEnvironment;

    #[test]
    fn parses_supported_environment_aliases() {
        assert_eq!(
            DeploymentEnvironment::parse("development").ok(),
            Some(DeploymentEnvironment::Dev)
        );
        assert_eq!(
            DeploymentEnvironment::parse("PRODUCTION").ok(),
            Some(DeploymentEnvironment::Prod)
        );
    }

    #[test]
    fn rejects_unknown_environment() {
        assert!(DeploymentEnvironment::parse("prodution").is_err());
        assert!(DeploymentEnvironment::parse("").is_err());
    }
}
