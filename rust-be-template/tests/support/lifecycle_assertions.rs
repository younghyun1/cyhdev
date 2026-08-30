//! Lifecycle assertions shared by database-backed account tests.

use chrono::{DateTime, Utc};
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use rust_be_template::{
    domain::{
        blog::blog::UserBadgeInfo,
        photography::photographs::Photograph,
    },
    features::accounts::{
        domain::account::DELETED_USER_DISPLAY_NAME,
        error::AccountError,
    },
    schema::{
        deleted_account_retention, email_verification_tokens, password_reset_tokens,
        user_profile_pictures, user_roles, users,
    },
};

use super::{
    database::{BoxError, TestResult, require},
    fixtures::{AccountTestContext, VALID_PASSWORD},
};

pub async fn require_login_rejected(
    context: &AccountTestContext,
    original_email: &str,
) -> TestResult {
    match context
        .accounts
        .login(original_email, VALID_PASSWORD, None)
        .await
    {
        Err(AccountError::InvalidCredentials) => Ok(()),
        Err(error) => Err(Box::new(error) as BoxError),
        Ok(_) => require(false, "deleted account authenticated with its original identity"),
    }
}

pub async fn retained_and_tombstone_identity(
    context: &AccountTestContext,
    user_id: Uuid,
) -> TestResult<(String, String, String, String)> {
    let mut connection = context.pool.get().await?;
    let retained = deleted_account_retention::table
        .filter(deleted_account_retention::deleted_account_retention_user_id.eq(user_id))
        .select((
            deleted_account_retention::deleted_account_retention_user_name,
            deleted_account_retention::deleted_account_retention_email,
        ))
        .first::<(String, String)>(&mut connection)
        .await?;
    let tombstone = users::table
        .find(user_id)
        .select((users::user_name, users::user_email))
        .first::<(String, String)>(&mut connection)
        .await?;
    drop(connection);
    Ok((retained.0, retained.1, tombstone.0, tombstone.1))
}

pub async fn require_account_authority_cleared(
    context: &AccountTestContext,
    user_id: Uuid,
) -> TestResult {
    let mut connection = context.pool.get().await?;
    let remaining = (
        user_roles::table
            .filter(user_roles::user_id.eq(user_id))
            .count()
            .get_result::<i64>(&mut connection)
            .await?,
        email_verification_tokens::table
            .filter(email_verification_tokens::user_id.eq(user_id))
            .count()
            .get_result::<i64>(&mut connection)
            .await?,
        password_reset_tokens::table
            .filter(password_reset_tokens::user_id.eq(user_id))
            .count()
            .get_result::<i64>(&mut connection)
            .await?,
    );
    drop(connection);
    require(remaining == (0, 0, 0), "soft deletion retained account authority")
}

pub async fn require_public_identity_is_generic(
    context: &AccountTestContext,
    user_id: Uuid,
    original_name: &str,
) -> TestResult {
    match context.accounts.public_account(original_name).await {
        Err(AccountError::AccountNotFound) => {}
        Err(error) => return Err(Box::new(error) as BoxError),
        Ok(_) => return require(false, "deleted account remained publicly discoverable"),
    }

    let badge = UserBadgeInfo::deleted();
    require(
        badge.user_name == DELETED_USER_DISPLAY_NAME
            && badge.user_profile_picture_url.is_empty()
            && badge.user_country_flag.is_none(),
        "deleted public badge disclosed account identity",
    )?;

    let mut connection = context.pool.get().await?;
    let mut photograph = rust_be_template::schema::photographs::table
        .filter(rust_be_template::schema::photographs::user_id.eq(user_id))
        .first::<Photograph>(&mut connection)
        .await?;
    drop(connection);
    photograph.anonymize_deleted_owner();
    require(
        photograph.user_id.is_nil()
            && photograph.photograph_lat == 0.0
            && photograph.photograph_lon == 0.0,
        "deleted photograph presentation retained owner or location identity",
    )
}

pub async fn require_permanent_tombstone(
    context: &AccountTestContext,
    user_id: Uuid,
    expected_name: &str,
    expected_email: &str,
) -> TestResult {
    let mut connection = context.pool.get().await?;
    let tombstone = users::table
        .find(user_id)
        .select((
            users::user_name,
            users::user_email,
            users::user_deleted_at,
            users::user_purge_after,
            users::user_hard_purged_at,
        ))
        .first::<(
            String,
            String,
            Option<DateTime<Utc>>,
            Option<DateTime<Utc>>,
            Option<DateTime<Utc>>,
        )>(&mut connection)
        .await?;
    let retention_rows = deleted_account_retention::table
        .filter(deleted_account_retention::deleted_account_retention_user_id.eq(user_id))
        .count()
        .get_result::<i64>(&mut connection)
        .await?;
    let profile_rows = user_profile_pictures::table
        .filter(user_profile_pictures::user_id.eq(user_id))
        .count()
        .get_result::<i64>(&mut connection)
        .await?;
    drop(connection);

    require(
        tombstone.0 == expected_name
            && tombstone.1 == expected_email
            && tombstone.2.is_some()
            && tombstone.3.is_some()
            && tombstone.4.is_some(),
        "hard purge did not leave the scheduled users tombstone",
    )?;
    require(
        retention_rows == 0 && profile_rows == 0,
        "hard purge retained private identity or finalized profile metadata",
    )
}
