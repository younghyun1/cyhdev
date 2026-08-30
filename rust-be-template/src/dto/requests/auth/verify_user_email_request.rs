use utoipa::ToSchema;

/// One-time token submitted only after the browser user confirms verification.
#[derive(serde_derive::Deserialize, ToSchema)]
pub struct VerifyUserEmailRequest {
    pub email_validation_token_id: uuid::Uuid,
}

#[cfg(test)]
mod tests {
    use super::VerifyUserEmailRequest;

    #[test]
    fn request_requires_a_json_uuid_token() -> Result<(), serde_json::Error> {
        let token = uuid::Uuid::new_v4();
        let request: VerifyUserEmailRequest = serde_json::from_value(serde_json::json!({
            "email_validation_token_id": token,
        }))?;

        assert_eq!(request.email_validation_token_id, token);
        assert!(
            serde_json::from_value::<VerifyUserEmailRequest>(serde_json::json!({
                "email_validation_token_id": "not-a-uuid",
            }))
            .is_err()
        );
        Ok(())
    }
}
