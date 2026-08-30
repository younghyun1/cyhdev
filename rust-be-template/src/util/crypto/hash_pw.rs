use anyhow::Result;
use argon2::{Argon2, PasswordHasher};
use zeroize::Zeroizing;

pub async fn hash_pw(password: Zeroizing<String>) -> Result<String> {
    tokio::task::spawn_blocking(move || {
        let argon2 = Argon2::default();
        argon2
            .hash_password(password.as_bytes())
            .map(|password_hash| password_hash.to_string())
            .map_err(|e| anyhow::anyhow!(e))
    })
    .await?
}
