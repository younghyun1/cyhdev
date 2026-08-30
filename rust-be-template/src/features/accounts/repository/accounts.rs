//! Read-only account queries.

use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper, dsl::exists};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    features::accounts::{
        domain::account::{CurrentAccount, LoginAccount, PublicAccount, SessionAccount},
        error::AccountError,
        repository::{
            account_repository::AccountRepository,
            records::{
                AccountProfileRecord, AccountRecord, ProfilePictureRecord, PublicAccountRecord,
            },
        },
    },
    schema::{user_profile_pictures, users},
};

impl AccountRepository {
    pub async fn email_exists(&self, email: &str) -> Result<bool, AccountError> {
        let mut connection = self.connection().await?;
        diesel::select(exists(users::table.filter(users::user_email.eq(email))))
            .get_result(&mut connection)
            .await
            .map_err(AccountError::Query)
    }

    pub async fn login_account_by_email(
        &self,
        email: &str,
    ) -> Result<Option<LoginAccount>, AccountError> {
        let mut connection = self.connection().await?;
        let record = users::table
            .filter(users::user_email.eq(email))
            .select(AccountRecord::as_select())
            .first::<AccountRecord>(&mut connection)
            .await
            .optional()
            .map_err(AccountError::Query)?;

        Ok(record.map(AccountRecord::into_login_account))
    }

    pub async fn session_account(
        &self,
        user_id: Uuid,
    ) -> Result<Option<SessionAccount>, AccountError> {
        let mut connection = self.connection().await?;
        users::table
            .filter(users::user_id.eq(user_id))
            .select(AccountProfileRecord::as_select())
            .first::<AccountProfileRecord>(&mut connection)
            .await
            .optional()
            .map(|record| record.map(SessionAccount::from))
            .map_err(AccountError::Query)
    }

    pub async fn current_account(
        &self,
        user_id: Uuid,
    ) -> Result<Option<CurrentAccount>, AccountError> {
        let mut connection = self.connection().await?;
        let profile = users::table
            .filter(users::user_id.eq(user_id))
            .select(AccountProfileRecord::as_select())
            .first::<AccountProfileRecord>(&mut connection)
            .await
            .optional()
            .map_err(AccountError::Query)?;

        let profile = match profile {
            Some(profile) => profile.into(),
            None => return Ok(None),
        };

        let profile_picture = user_profile_pictures::table
            .filter(user_profile_pictures::user_id.eq(user_id))
            .order(user_profile_pictures::user_profile_picture_created_at.desc())
            .select(ProfilePictureRecord::as_select())
            .first::<ProfilePictureRecord>(&mut connection)
            .await
            .optional()
            .map_err(AccountError::Query)?
            .map(Into::into);

        Ok(Some(CurrentAccount {
            profile,
            profile_picture,
        }))
    }

    pub async fn public_account_by_user_name(
        &self,
        user_name: &str,
    ) -> Result<Option<PublicAccount>, AccountError> {
        let mut connection = self.connection().await?;
        let account = users::table
            .filter(users::user_name.eq(user_name))
            .select(PublicAccountRecord::as_select())
            .first::<PublicAccountRecord>(&mut connection)
            .await
            .optional()
            .map_err(AccountError::Query)?;
        let account = match account {
            Some(account) => account,
            None => return Ok(None),
        };

        let profile_picture_url = user_profile_pictures::table
            .filter(user_profile_pictures::user_id.eq(account.user_id()))
            .filter(user_profile_pictures::user_profile_picture_is_on_cloud.eq(true))
            .filter(user_profile_pictures::user_profile_picture_link.is_not_null())
            .order(user_profile_pictures::user_profile_picture_created_at.desc())
            .select(user_profile_pictures::user_profile_picture_link)
            .first::<Option<String>>(&mut connection)
            .await
            .optional()
            .map_err(AccountError::Query)?
            .flatten();

        Ok(Some(account.into_public_account(profile_picture_url)))
    }
}
