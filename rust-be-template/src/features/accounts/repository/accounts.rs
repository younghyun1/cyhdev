//! Read-only account queries.

use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, NullableExpressionMethods,
    OptionalExtension, QueryDsl, SelectableHelper, dsl::exists,
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

    /// Reads the profile and its active picture in one statement. The partial
    /// unique index `user_profile_pictures_one_active_per_user` admits at most
    /// one active picture, so the left join cannot multiply the account row.
    pub async fn current_account(
        &self,
        user_id: Uuid,
    ) -> Result<Option<CurrentAccount>, AccountError> {
        let mut connection = self.connection().await?;
        let record = users::table
            .left_join(
                user_profile_pictures::table.on(user_profile_pictures::user_id
                    .eq(users::user_id)
                    .and(user_profile_pictures::user_profile_picture_is_active.eq(true))),
            )
            .filter(users::user_id.eq(user_id))
            .filter(users::user_deleted_at.is_null())
            .select((
                AccountProfileRecord::as_select(),
                Option::<ProfilePictureRecord>::as_select(),
            ))
            .first::<(AccountProfileRecord, Option<ProfilePictureRecord>)>(&mut connection)
            .await
            .optional()
            .map_err(AccountError::Query)?;
        Ok(record.map(|(profile, profile_picture)| CurrentAccount {
            profile: profile.into(),
            profile_picture: profile_picture.map(Into::into),
        }))
    }

    /// Reads a public profile and its picture link in one statement. The
    /// picture conditions sit in the join so an account without a qualifying
    /// picture is still returned, with no link.
    pub async fn public_account_by_user_name(
        &self,
        user_name: &str,
    ) -> Result<Option<PublicAccount>, AccountError> {
        let mut connection = self.connection().await?;
        let record = users::table
            .left_join(
                user_profile_pictures::table.on(user_profile_pictures::user_id
                    .eq(users::user_id)
                    .and(user_profile_pictures::user_profile_picture_is_active.eq(true))
                    .and(user_profile_pictures::user_profile_picture_is_on_cloud.eq(true))
                    .and(user_profile_pictures::user_profile_picture_link.is_not_null())),
            )
            .filter(lower(users::user_name).eq(lower(user_name)))
            .filter(users::user_deleted_at.is_null())
            .select((
                PublicAccountRecord::as_select(),
                user_profile_pictures::user_profile_picture_link.nullable(),
            ))
            .first::<(PublicAccountRecord, Option<String>)>(&mut connection)
            .await
            .optional()
            .map_err(AccountError::Query)?;
        Ok(record
            .map(|(account, profile_picture_url)| account.into_public_account(profile_picture_url)))
    }
}
