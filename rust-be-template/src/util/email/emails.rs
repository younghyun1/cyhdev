use crate::DOMAIN_NAME;
use lettre::message::Mailbox;
use tracing::error;

pub const PASSWORD_RESET_EMAIL: &str = include_str!("./password_reset.html");
pub const VALIDATE_EMAIL_EMAIL: &str = include_str!("./validate_email.html");

pub struct PasswordResetEmail {
    pub email: String,
}

impl Default for PasswordResetEmail {
    fn default() -> Self {
        Self::new()
    }
}

impl PasswordResetEmail {
    pub fn new() -> Self {
        PasswordResetEmail {
            email: PASSWORD_RESET_EMAIL.to_string(),
        }
    }

    pub fn set_link(mut self, link: &str) -> Self {
        self.email = self.email.replace("$1", link);
        self
    }

    pub fn to_message(self, user_email: &str) -> anyhow::Result<lettre::Message> {
        let from = parse_mailbox("cyhdev.com <donotreply@cyhdev.com>", "from")?;
        let to = parse_mailbox(user_email, "to")?;
        match lettre::Message::builder()
            .from(from)
            .to(to)
            .subject("Reset Your Password")
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(self.email)
        {
            Ok(message) => Ok(message),
            Err(e) => {
                error!(error = %e, "Failed to build password reset email");
                Err(e.into())
            }
        }
    }
}

pub struct ValidateEmailEmail {
    pub email: String,
}

impl Default for ValidateEmailEmail {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidateEmailEmail {
    pub fn new() -> Self {
        ValidateEmailEmail {
            email: VALIDATE_EMAIL_EMAIL.to_string(),
        }
    }

    pub fn set_fields(
        mut self,
        valid_until: chrono::DateTime<chrono::Utc>,
        token_id: uuid::Uuid,
        public_app_origin: &str,
    ) -> Self {
        self.email = self
            .email
            .replace(
                "$1",
                &verification_confirmation_link(public_app_origin, token_id),
            )
            .replace("$2", &valid_until.to_string());
        self
    }

    pub fn to_message(self, user_email: &str) -> anyhow::Result<lettre::Message> {
        let from_raw = format!("cyhdev.com <donotreply@{DOMAIN_NAME}>");
        let from = parse_mailbox(&from_raw, "from")?;
        let to = parse_mailbox(user_email, "to")?;
        match lettre::Message::builder()
            .from(from)
            .to(to)
            .subject("Validate Your Email")
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(self.email)
        {
            Ok(message) => Ok(message),
            Err(e) => {
                error!(error = %e, "Failed to build validation email");
                Err(e.into())
            }
        }
    }
}

/// Builds the same-origin SPA link without exposing the token to HTTP servers or referrers.
fn verification_confirmation_link(public_app_origin: &str, token_id: uuid::Uuid) -> String {
    format!("{public_app_origin}/verify-email#token={token_id}")
}

fn parse_mailbox(raw: &str, field: &'static str) -> anyhow::Result<Mailbox> {
    match raw.parse::<Mailbox>() {
        Ok(mailbox) => Ok(mailbox),
        Err(e) => {
            error!(field, error = %e, "Failed to parse email mailbox");
            Err(e.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::verification_confirmation_link;

    #[test]
    fn verification_link_targets_the_spa_with_a_fragment_token() {
        let token = uuid::Uuid::new_v4();
        let link = verification_confirmation_link("https://app.example.test", token);

        assert_eq!(
            link,
            format!("https://app.example.test/verify-email#token={token}")
        );
        assert!(!link.contains("/api/"));
        assert!(!link.contains('?'));
    }
}
