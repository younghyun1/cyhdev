use utoipa::ToSchema;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// One-time token submitted only after the browser user confirms verification.
#[derive(serde_derive::Deserialize, Zeroize, ZeroizeOnDrop, ToSchema)]
pub struct VerifyUserEmailRequest {
    /// Capability from the emailed link fragment: 43 unpadded base64url characters.
    #[schema(min_length = 43, max_length = 43, pattern = "^[A-Za-z0-9_-]{43}$")]
    pub email_verification_token: String,
}

#[cfg(test)]
mod tests {
    use super::VerifyUserEmailRequest;

    #[test]
    fn request_carries_the_token_as_opaque_text() -> Result<(), serde_json::Error> {
        let token = "A".repeat(43);
        let request: VerifyUserEmailRequest = serde_json::from_value(serde_json::json!({
            "email_verification_token": token,
        }))?;

        assert_eq!(request.email_verification_token, token);
        assert!(
            serde_json::from_value::<VerifyUserEmailRequest>(serde_json::json!({
                "email_verification_token": 7,
            }))
            .is_err()
        );
        Ok(())
    }
}
