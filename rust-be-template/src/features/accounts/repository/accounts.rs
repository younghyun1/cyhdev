//! Read-only account queries.

use diesel::{
    ExpressionMethods, NullableExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper,
    dsl::exists,
};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    features::accounts::{
        domain::{
            account::{CurrentAccount, LoginCandidate, PublicAccount, SessionAccount},
            role::RoleType,
        },
        error::AccountError,
        repository::{
            account_repository::AccountRepository,
            records::{
                AccountProfileRecord, AccountRecord, ProfilePictureRecord, PublicAccountRecord,
            },
            sql_functions::lower,
        },
    },
    schema::{user_profile_pictures, user_roles, users},
};

impl AccountRepository {
    pub async fn email_exists(&self, email: &str) -> Result<bool, AccountError> {
        let mut connection = self.connection().await?;
        diesel::select(exists(
            users::table
                .filter(lower(users::user_email).eq(lower(email)))
                .filter(users::user_deleted_at.is_null()),
        ))
        .get_result(&mut connection)
        .await
        .map_err(AccountError::Query)
    }

    /// Reads the credentials and the role in one round trip, so a successful login needs no
    /// second query before creating its session.
    pub async fn login_account_by_email(
        &self,
        email: &str,
    ) -> Result<Option<LoginCandidate>, AccountError> {
        let mut connection = self.connection().await?;
        let record = users::table
            .left_join(user_roles::table)
            .filter(lower(users::user_email).eq(lower(email)))
            .filter(users::user_deleted_at.is_null())
            .select((AccountRecord::as_select(), user_roles::role_id.nullable()))
            .first::<(AccountRecord, Option<Uuid>)>(&mut connection)
            .await
            .optional()
            .map_err(AccountError::Query)?;
        let (record, role_id) = match record {
            Some(record) => record,
            None => return Ok(None),
        };
        let role = match role_id {
            Some(role_id) => {
                Some(RoleType::from_uuid(role_id).ok_or(AccountError::InvalidRoleId(role_id))?)
            }
            None => None,
        };
        Ok(Some(LoginCandidate {
            account: record.into_login_account(),
            role,
        }))
    }

    pub async fn session_account(
        &self,
        user_id: Uuid,
    ) -> Result<Option<SessionAccount>, AccountError> {
        let mut connection = self.connection().await?;
        users::table
            .filter(users::user_id.eq(user_id))
            .filter(users::user_deleted_at.is_null())
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
            .filter(users::user_deleted_at.is_null())
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
            .filter(user_profile_pictures::user_profile_picture_is_active.eq(true))
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
            .filter(lower(users::user_name).eq(lower(user_name)))
            .filter(users::user_deleted_at.is_null())
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
            .filter(user_profile_pictures::user_profile_picture_is_active.eq(true))
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
