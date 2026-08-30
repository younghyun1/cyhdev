//! Atomic full-profile updates for active accounts.

use chrono::{DateTime, Utc};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper, dsl::exists};
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    features::accounts::{
        domain::account::{AccountProfile, ProfileUpdateCommand},
        error::AccountError,
        repository::{
            account_repository::AccountRepository, records::AccountProfileRecord,
        },
    },
    schema::{iso_country, iso_country_subdivision, iso_language, users},
};

const USER_NAME_UNIQUE_CONSTRAINT: &str = "users_user_name_unique";

impl AccountRepository {
    pub async fn update_profile(
        &self,
        user_id: Uuid,
        expected_password_hash: &str,
        command: &ProfileUpdateCommand,
        updated_at: DateTime<Utc>,
    ) -> Result<AccountProfile, AccountError> {
        let mut connection = self.connection().await?;
        let result = connection
            .transaction::<AccountProfile, AccountError, _>(async move |connection| {
                let credentials = users::table
                    .filter(users::user_id.eq(user_id))
                    .filter(users::user_deleted_at.is_null())
                    .select((users::user_password_hash, users::user_is_system_actor))
                    .for_update()
                    .first::<(String, bool)>(&mut *connection)
                    .await
                    .optional()?
                    .ok_or(AccountError::AccountNotFound)?;
                if credentials.1 {
                    return Err(AccountError::SystemActorProtected);
                }
                if credentials.0 != expected_password_hash {
                    return Err(AccountError::AccountChanged);
                }

                let subdivision_id = command.subdivision.unwrap_or(i32::MIN);
                let (country_exists, language_exists, subdivision_exists) = diesel::select((
                    exists(
                        iso_country::table
                            .filter(iso_country::country_code.eq(command.country))
                            .filter(iso_country::is_country.eq(true)),
                    ),
                    exists(
                        iso_language::table
                            .filter(iso_language::language_code.eq(command.language)),
                    ),
                    exists(
                        iso_country_subdivision::table
                            .filter(iso_country_subdivision::subdivision_id.eq(subdivision_id))
                            .filter(iso_country_subdivision::country_code.eq(command.country)),
                    ),
                ))
                .get_result::<(bool, bool, bool)>(&mut *connection)
                .await?;
                if !country_exists
                    || !language_exists
                    || (command.subdivision.is_some() && !subdivision_exists)
                {
                    return Err(AccountError::InvalidAccountGeography);
                }

                diesel::update(users::table.filter(users::user_id.eq(user_id)))
                    .set((
                        users::user_name.eq(&command.user_name),
                        users::user_country.eq(command.country),
                        users::user_language.eq(command.language),
                        users::user_subdivision.eq(command.subdivision),
                        users::user_updated_at.eq(updated_at),
                    ))
                    .returning(AccountProfileRecord::as_returning())
                    .get_result::<AccountProfileRecord>(&mut *connection)
                    .await
                    .map(Into::into)
                    .map_err(AccountError::Mutation)
            })
            .await;
        result.map_err(classify_profile_update_error)
    }
}

fn classify_profile_update_error(error: AccountError) -> AccountError {
    let duplicate_user_name = match &error {
        AccountError::Mutation(diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::UniqueViolation,
            information,
        )) => information.constraint_name() == Some(USER_NAME_UNIQUE_CONSTRAINT),
        _ => false,
    };
    if duplicate_user_name {
        match error {
            AccountError::Mutation(source) => AccountError::DuplicateUserName(source),
            error => error,
        }
    } else {
        error
    }
}
