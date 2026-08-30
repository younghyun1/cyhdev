mod support;

use zeroize::Zeroizing;

use rust_be_template::features::accounts::{
    domain::account::{SignupCommand, SignupReceipt},
    error::AccountError,
};

use support::{
    database::{DatabaseTestFuture, TestDatabase, TestResult, require, run_database_test},
    fixtures::{VALID_PASSWORD, account_test_context, seed_account},
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn concurrent_signup_maps_named_identity_constraints() -> TestResult {
    run_database_test(concurrent_identity_case).await
}

fn concurrent_identity_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let fixture = seed_account(&context, "IdentitySeed").await?;

        let email_a = context.accounts.signup(signup_command(
            "ConcurrentEmailA",
            "concurrent@example.test",
            fixture.country,
            fixture.language,
        ));
        let email_b = context.accounts.signup(signup_command(
            "ConcurrentEmailB",
            "concurrent@example.test",
            fixture.country,
            fixture.language,
        ));
        let (email_a, email_b) = tokio::join!(email_a, email_b);
        require(
            is_single_email_conflict(&email_a, &email_b),
            "concurrent email signup did not yield one success and one email conflict",
        )?;

        let name_a = context.accounts.signup(signup_command(
            "ConcurrentName",
            "concurrent-name-a@example.test",
            fixture.country,
            fixture.language,
        ));
        let name_b = context.accounts.signup(signup_command(
            "ConcurrentName",
            "concurrent-name-b@example.test",
            fixture.country,
            fixture.language,
        ));
        let (name_a, name_b) = tokio::join!(name_a, name_b);
        require(
            is_single_user_name_conflict(&name_a, &name_b),
            "concurrent username signup did not yield one success and one username conflict",
        )?;

        let public_account = context.accounts.public_account(&fixture.user_name).await?;
        require(
            public_account.user_id == fixture.user_id,
            "public account lookup returned the wrong unique account",
        )
    })
}

fn signup_command(user_name: &str, user_email: &str, country: i32, language: i32) -> SignupCommand {
    SignupCommand {
        user_name: user_name.to_owned(),
        user_email: user_email.to_owned(),
        password: Zeroizing::new(VALID_PASSWORD.to_owned()),
        country,
        language,
        subdivision: None,
    }
}

fn is_single_email_conflict(
    first: &Result<SignupReceipt, AccountError>,
    second: &Result<SignupReceipt, AccountError>,
) -> bool {
    matches!(
        (first, second),
        (Ok(_), Err(AccountError::DuplicateEmail(_)))
            | (Err(AccountError::DuplicateEmail(_)), Ok(_))
    )
}

fn is_single_user_name_conflict(
    first: &Result<SignupReceipt, AccountError>,
    second: &Result<SignupReceipt, AccountError>,
) -> bool {
    matches!(
        (first, second),
        (Ok(_), Err(AccountError::DuplicateUserName(_)))
            | (Err(AccountError::DuplicateUserName(_)), Ok(_))
    )
}
