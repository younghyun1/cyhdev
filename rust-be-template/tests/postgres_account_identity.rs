mod support;

use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

use rust_be_template::{
    features::accounts::{domain::account::SignupOutcome, error::AccountError},
    schema::users,
};

use support::{
    database::{
        BoxError, DatabaseTestFuture, TestDatabase, TestResult, require, run_database_test,
    },
    fixtures::{
        AccountTestContext, VALID_PASSWORD, account_test_context, seed_account,
        seed_verified_account, signup_command,
    },
};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn concurrent_signup_maps_named_identity_constraints() -> TestResult {
    run_database_test(concurrent_identity_case).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires explicit TEST_DATABASE_URL and PostgreSQL 18"]
async fn identities_are_unique_and_matched_without_letter_case() -> TestResult {
    run_database_test(case_insensitive_case).await
}

fn concurrent_identity_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let fixture = seed_account(&context, "IdentitySeed").await?;

        let email_a = context.accounts.signup(signup_command(
            "ConcurrentEmailA",
            "concurrent@example.test",
            VALID_PASSWORD,
            &fixture,
        ));
        let email_b = context.accounts.signup(signup_command(
            "ConcurrentEmailB",
            "concurrent@example.test",
            VALID_PASSWORD,
            &fixture,
        ));
        let (email_a, email_b) = tokio::join!(email_a, email_b);
        require(
            matches!(
                (&email_a, &email_b),
                (
                    Ok(SignupOutcome::Registered(_)),
                    Ok(SignupOutcome::ReplacedUnverified(_))
                ) | (
                    Ok(SignupOutcome::ReplacedUnverified(_)),
                    Ok(SignupOutcome::Registered(_))
                )
            ),
            "concurrent email signup did not register once and replace once",
        )?;
        require(
            accounts_with_email(&context, "concurrent@example.test").await? == 1,
            "concurrent email signup created more than one account",
        )?;

        let name_a = context.accounts.signup(signup_command(
            "ConcurrentName",
            "concurrent-name-a@example.test",
            VALID_PASSWORD,
            &fixture,
        ));
        let name_b = context.accounts.signup(signup_command(
            "ConcurrentName",
            "concurrent-name-b@example.test",
            VALID_PASSWORD,
            &fixture,
        ));
        let (name_a, name_b) = tokio::join!(name_a, name_b);
        require(
            matches!(
                (&name_a, &name_b),
                (Ok(_), Err(AccountError::DuplicateUserName(_)))
                    | (Err(AccountError::DuplicateUserName(_)), Ok(_))
            ),
            "concurrent username signup did not yield one success and one username conflict",
        )?;

        let public_account = context.accounts.public_account(&fixture.user_name).await?;
        require(
            public_account.user_id == fixture.user_id,
            "public account lookup returned the wrong unique account",
        )
    })
}

fn case_insensitive_case(database: &TestDatabase) -> DatabaseTestFuture<'_> {
    Box::pin(async move {
        let context = account_test_context(database)?;
        let fixture = seed_verified_account(&context, "CaseOwner").await?;
        let login = context
            .accounts
            .login(&fixture.email.to_uppercase(), VALID_PASSWORD, None)
            .await?;
        require(
            login.user_id == fixture.user_id,
            "email lookup depended on letter case",
        )?;
        match context
            .accounts
            .signup(signup_command(
                &fixture.user_name.to_uppercase(),
                "case-other@example.test",
                VALID_PASSWORD,
                &fixture,
            ))
            .await
        {
            Err(AccountError::DuplicateUserName(_)) => {}
            Err(error) => return Err(Box::new(error) as BoxError),
            Ok(_) => return require(false, "user names differing by case were both accepted"),
        }
        let mixed = context
            .accounts
            .signup(signup_command(
                "MixedCaseName",
                "Mixed.Case@Example.TEST",
                VALID_PASSWORD,
                &fixture,
            ))
            .await?;
        require(
            matches!(mixed, SignupOutcome::Registered(ref receipt)
                if receipt.user_email == "mixed.case@example.test"),
            "signup did not store the email in lowercase",
        )?;
        let public = context.accounts.public_account("mixedcasename").await?;
        require(
            public.user_name == "MixedCaseName",
            "user name did not keep its display case",
        )
    })
}

async fn accounts_with_email(context: &AccountTestContext, email: &str) -> TestResult<i64> {
    let mut connection = context.pool.get().await?;
    Ok(users::table
        .filter(users::user_email.eq(email))
        .count()
        .get_result::<i64>(&mut connection)
        .await?)
}
