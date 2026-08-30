use anyhow::Result;
use argon2::{
    Argon2, PasswordVerifier,
    password_hash::{Error as PasswordHashError, phc::PasswordHash},
};
use zeroize::Zeroizing;

pub async fn verify_pw(password: &str, expected_hash: &str) -> Result<bool> {
    let password = Zeroizing::new(password.to_owned());
    let expected_hash = expected_hash.to_owned();
    tokio::task::spawn_blocking(move || {
        let argon2 = Argon2::default();

        let parsed_hash = PasswordHash::new(&expected_hash).map_err(|e| anyhow::anyhow!(e))?;

        match argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(PasswordHashError::PasswordInvalid) => Ok(false),
            Err(e) => Err(anyhow::anyhow!(e)),
        }
    })
    .await?
}
