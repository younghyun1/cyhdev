use zeroize::Zeroizing;

use super::{hash_pw::hash_pw, verify_pw::verify_pw};

#[tokio::test]
async fn generated_hash_accepts_only_the_original_password() -> anyhow::Result<()> {
    let password_hash = hash_pw(Zeroizing::new("A-valid-password-7".to_owned())).await?;

    assert!(verify_pw("A-valid-password-7", &password_hash).await?);
    assert!(!verify_pw("A-different-password-8", &password_hash).await?);
    Ok(())
}

#[tokio::test]
async fn malformed_hash_is_an_error_instead_of_a_password_mismatch() {
    assert!(
        verify_pw("A-valid-password-7", "not-a-phc-string")
            .await
            .is_err()
    );
}
