use anyhow::Result;
use argon2::{Argon2, PasswordHasher};
use zeroize::Zeroizing;

pub async fn hash_pw(password: Zeroizing<String>) -> Result<String> {
    tokio::task::spawn_blocking(move || hash_pw_blocking(password.as_bytes())).await?
}

/// Hashes on the current thread; callers must already be off the async runtime workers.
pub fn hash_pw_blocking(password: &[u8]) -> Result<String> {
    Argon2::default()
        .hash_password(password)
        .map(|password_hash| password_hash.to_string())
        .map_err(|e| anyhow::anyhow!(e))
}
