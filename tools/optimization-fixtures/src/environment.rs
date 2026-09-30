//! Private shell environment containing synthetic credentials only.
use std::os::unix::fs::PermissionsExt;
use std::{fs, path::Path};

pub fn write(runtime: &Path) -> anyhow::Result<()> {
    let root = runtime
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("runtime must be UTF-8"))?;
    let database = std::env::var("DB_URL")?;
    let mut values = vec![
        ("CYHDEV_OPT_DISPOSABLE", "1".to_owned()),
        ("DB_URL", database),
        (
            "CYHDEV_OPT_MEMBER_PASSWORD",
            "OptimizationFixture123".to_owned(),
        ),
        (
            "CYHDEV_OPT_ADMIN_PASSWORD",
            "OptimizationFixture123".to_owned(),
        ),
        ("AWS_SES_SMTP_URL", "127.0.0.1".to_owned()),
        (
            "AWS_SES_SMTP_USERNAME",
            "optimization-fixture-mail".to_owned(),
        ),
        (
            "AWS_SES_SMTP_ACCESS_KEY",
            "optimization-fixture-mail".to_owned(),
        ),
        ("LOCAL_SMTP_PORT", crate::files::SMTP_PORT.to_string()),
        (
            "AWS_ENDPOINT_URL",
            format!("http://127.0.0.1:{}", crate::files::HTTP_PORT),
        ),
        (
            "AWS_IMAGE_UPLOAD_KEY",
            "optimization-fixture-image".to_owned(),
        ),
        (
            "AWS_IMAGE_UPLOAD_SECRET_KEY",
            "optimization-fixture-image".to_owned(),
        ),
        ("AWS_REGION", "us-west-1".to_owned()),
        (
            "OIDC_ISSUER_URL",
            format!("http://127.0.0.1:{}/", crate::files::HTTP_PORT),
        ),
        ("OIDC_PROVIDER_NAME", "Optimization fixture".to_owned()),
        ("OIDC_CLIENT_ID", "optimization-fixture-client".to_owned()),
        ("OIDC_CLIENT_SECRET", "optimization-fixture-oidc".to_owned()),
        ("RTC_TURN_URL", String::new()),
        ("RTC_TURN_USER", String::new()),
        ("RTC_TURN_PASS", String::new()),
        (
            "APP_NAME_VERSION",
            "cyhdev optimization campaign".to_owned(),
        ),
        ("RTC_ENABLE", "true".to_owned()),
        ("RTC_PUBLIC_IP", "127.0.0.1".to_owned()),
        ("RTC_UDP_PORT_START", "35900".to_owned()),
        ("RTC_MAX_PARTICIPANTS", "4".to_owned()),
        ("RUST_LOG", "info".to_owned()),
    ];
    for (name, path) in [
        ("CERT_CHAIN_DIR", "tls-cert.pem"),
        ("PRIV_KEY_DIR", "tls-key.pem"),
        ("LOCAL_SMTP_CA_PEM", "tls-cert.pem"),
        ("MINECRAFT_WORLD_SOCKET", "world.sock"),
        ("MINECRAFT_MAP_CONTROL_SOCKET", "control.sock"),
        ("SQUAREMAP_WEB_DIR", "squaremap"),
        ("SEARCH_INDEX_PATH", "data/search_index"),
    ] {
        values.push((name, format!("{root}/{path}")));
    }
    let mut script = String::new();
    for (name, value) in values {
        anyhow::ensure!(
            !value.contains(['\n', '\r', '\0']),
            "invalid environment value"
        );
        script.push_str(&format!(
            "export {name}='{}'\n",
            value.replace('\'', "'\\''")
        ));
    }
    let path = runtime.join("environment.sh");
    fs::write(&path, script)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}
