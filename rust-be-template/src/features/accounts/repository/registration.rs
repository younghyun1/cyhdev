//! Atomic account registration persistence.

use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, dsl::exists};
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    features::accounts::{
        domain::{
            account::{DuplicateRegistration, NewAccountRegistration},
            role::RoleType,
        },
        error::AccountError,
        repository::{
            account_repository::AccountRepository,
            records::{NewAccountRecord, NewEmailVerificationTokenRecord, NewUserRoleRecord},
            sql_functions::lower,
        },
    },
    schema::{email_verification_tokens, user_roles, users},
};

impl AccountRepository {
    pub async fn register_account(
        &self,
        registration: &NewAccountRegistration,
    ) -> Result<(), AccountError> {
        let mut connection = self.connection().await?;
        let account = &registration.account;
        let new_account = NewAccountRecord {
            user_name: &account.user_name,
            user_email: &account.user_email,
            user_password_hash: &account.password_hash,
            user_country: account.country,
            user_language: account.language,
            user_subdivision: account.subdivision,
        };

        let transaction_result = connection
            .transaction::<(), diesel::result::Error, _>(async |connection| {
                let user_id = diesel::insert_into(users::table)
                    .values(new_account)
                    .returning(users::user_id)
                    .get_result(&mut *connection)
                    .await?;

                diesel::insert_into(user_roles::table)
                    .values(NewUserRoleRecord {
                        user_id,
                        role_id: RoleType::User.id(),
                    })
                    .execute(&mut *connection)
                    .await?;

                insert_verification_token(connection, user_id, registration).await
            })
            .await;

        transaction_result.map_err(classify_registration_error)
    }

    /// Settles a signup whose email already exists.
    ///
    /// An unverified account takes the new submission's password hash, user name, and
    /// geography, and every earlier verification token is replaced, so only the newest
    /// submitter can finish verification. A verified account is never changed. Either way a
    /// user name held by another account reports `DuplicateUserName`, the same result a
    /// fresh email would get, so the response does not reveal whether the email exists.
    pub async fn replace_unverified_registration(
        &self,
        registration: &NewAccountRegistration,
    ) -> Result<DuplicateRegistration, AccountError> {
        let mut connection = self.connection().await?;
        let account = &registration.account;
        let result = connection
            .transaction::<DuplicateRegistration, AccountError, _>(async move |connection| {
                let existing = users::table
                    .filter(lower(users::user_email).eq(lower(&account.user_email)))
                    .filter(users::user_deleted_at.is_null())
                    .select((users::user_id, users::user_is_email_verified))
                    .for_update()
                    .first::<(Uuid, bool)>(&mut *connection)
                    .await
                    .optional()?;
                let user_id = match existing {
                    Some((user_id, false)) => user_id,
                    Some((_, true)) | None => {
                        let name_taken = diesel::select(exists(
                            users::table
                                .filter(lower(users::user_name).eq(lower(&account.user_name))),
                        ))
                        .get_result::<bool>(&mut *connection)
                        .await?;
                        return if name_taken {
                            Err(AccountError::UserNameUnavailable)
                        } else {
                            Ok(DuplicateRegistration::Unchanged)
                        };
                    }
                };
                diesel::update(users::table.filter(users::user_id.eq(user_id)))
                    .set((
                        users::user_name.eq(&account.user_name),
                        users::user_password_hash.eq(&account.password_hash),
                        users::user_country.eq(account.country),
                        users::user_language.eq(account.language),
                        users::user_subdivision.eq(account.subdivision),
                        users::user_updated_at.eq(registration.verification_created_at),
                    ))
                    .execute(&mut *connection)
                    .await?;
                diesel::delete(
                    email_verification_tokens::table
                        .filter(email_verification_tokens::user_id.eq(user_id)),
                )
                .execute(&mut *connection)
                .await?;
                insert_verification_token(connection, user_id, registration).await?;
                Ok(DuplicateRegistration::ReplacedUnverified { user_id })
            })
            .await;
        result.map_err(|error| match error {
            AccountError::Mutation(error) => classify_registration_error(error),
            error => error,
        })
    }
}

async fn insert_verification_token(
    connection: &mut diesel_async::AsyncPgConnection,
    user_id: Uuid,
    registration: &NewAccountRegistration,
) -> Result<(), diesel::result::Error> {
    diesel::insert_into(email_verification_tokens::table)
        .values(NewEmailVerificationTokenRecord {
            user_id,
            email_verification_token_hash: registration.verification_digest.as_bytes(),
            email_verification_token_expires_at: registration.verification_expires_at,
            email_verification_token_created_at: registration.verification_created_at,
        })
        .execute(&mut *connection)
        .await
        .map(|_| ())
}

pub(super) const EMAIL_UNIQUE_CONSTRAINT: &str = "users_user_email_lower_unique";
pub(super) const USER_NAME_UNIQUE_CONSTRAINT: &str = "users_user_name_lower_unique";

#[derive(Clone, Copy)]
enum RegistrationConflict {
    Email,
    UserName,
    Other,
}

fn classify_registration_error(error: diesel::result::Error) -> AccountError {
    let conflict = match &error {
        diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::UniqueViolation,
            information,
        ) => match information.constraint_name() {
            Some(EMAIL_UNIQUE_CONSTRAINT) => RegistrationConflict::Email,
            Some(USER_NAME_UNIQUE_CONSTRAINT) => RegistrationConflict::UserName,
            _ => RegistrationConflict::Other,
        },
        _ => RegistrationConflict::Other,
    };

    match conflict {
        RegistrationConflict::Email => AccountError::DuplicateEmail(error),
        RegistrationConflict::UserName => AccountError::DuplicateUserName(error),
        RegistrationConflict::Other => AccountError::Mutation(error),
    }
}
