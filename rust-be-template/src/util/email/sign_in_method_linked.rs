//! Notice sent to an account owner when an external sign-in method is linked.

use jiff::Timestamp;
use lettre::message::Mailbox;

use crate::{DOMAIN_NAME, util::time::utc::rfc3339_utc};

const TEMPLATE: &str = include_str!("sign_in_method_linked.html");

pub struct SignInMethodLinkedEmail {
    body: String,
}

impl SignInMethodLinkedEmail {
    /// `provider_name` comes from deployment configuration and is HTML-escaped anyway.
    pub fn new(provider_name: &str, linked_at: Timestamp) -> Self {
        Self {
            body: TEMPLATE
                .replace("$1", &escape_html(provider_name))
                .replace("$2", &rfc3339_utc(linked_at)),
        }
    }

    pub fn to_message(self, owner_email: &str) -> anyhow::Result<lettre::Message> {
        let from = format!("cyhdev.com <donotreply@{DOMAIN_NAME}>").parse::<Mailbox>()?;
        let to = owner_email.parse::<Mailbox>()?;
        Ok(lettre::Message::builder()
            .from(from)
            .to(to)
            .subject("A new sign-in method was linked to your account")
            .header(lettre::message::header::ContentType::TEXT_HTML)
            .body(self.body)?)
    }
}

fn escape_html(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            other => escaped.push(other),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use jiff::Timestamp;

    use super::SignInMethodLinkedEmail;

    #[test]
    fn notice_names_the_escaped_provider_and_time() -> anyhow::Result<()> {
        let linked_at = "2026-09-23T12:00:00Z".parse::<Timestamp>()?;
        let email = SignInMethodLinkedEmail::new("Acme <ID>", linked_at);
        assert!(email.body.contains("Acme &lt;ID&gt;"));
        assert!(email.body.contains("2026-09-23T12:00:00+00:00"));
        assert!(!email.body.contains("Acme <ID>"));
        let message = email.to_message("owner@example.test")?;
        assert!(!message.formatted().is_empty());
        Ok(())
    }
}
