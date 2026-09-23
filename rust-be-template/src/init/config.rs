use anyhow::anyhow;
use lettre::transport::smtp::authentication::Credentials;

pub struct EmailConfig {
    smtp_url: String,
    smtp_username: String,
    smtp_password: String,
}

impl EmailConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let smtp_url = std::env::var("AWS_SES_SMTP_URL")
            .map_err(|_| anyhow!("Environment variable AWS_SES_SMTP_URL not found"))?;
        let smtp_username = std::env::var("AWS_SES_SMTP_USERNAME")
            .map_err(|_| anyhow!("Environment variable AWS_SES_SMTP_USERNAME not found"))?;
        let smtp_password = std::env::var("AWS_SES_SMTP_ACCESS_KEY")
            .map_err(|_| anyhow!("Environment variable AWS_SES_SMTP_ACCESS_KEY not found"))?;

        Ok(EmailConfig {
            smtp_url,
            smtp_username,
            smtp_password,
        })
    }

    pub fn to_creds(&self) -> Credentials {
        Credentials::new(self.smtp_username.clone(), self.smtp_password.clone())
    }

    pub fn get_url(&self) -> String {
        self.smtp_url.clone()
    }
}
