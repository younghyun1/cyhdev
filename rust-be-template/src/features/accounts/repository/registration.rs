//! Atomic account registration persistence.

use chrono::{DateTime, Utc};
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl};
use diesel_async::{AsyncConnection, RunQueryDsl};

use crate::{
    features::accounts::{
        domain::{account::NewAccountRegistration, role::RoleType},
        error::AccountError,
        repository::{
            account_repository::AccountRepository,
            records::{NewAccountRecord, NewEmailVerificationTokenRecord, NewUserRoleRecord},
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

                diesel::insert_into(email_verification_tokens::table)
                    .values(NewEmailVerificationTokenRecord {
                        user_id,
                        email_verification_token: registration.verification_token,
                        email_verification_token_expires_at: registration.verification_expires_at,
                        email_verification_token_created_at: registration.verification_created_at,
                    })
                    .execute(&mut *connection)
                    .await?;

                Ok(())
            })
            .await;

        transaction_result.map_err(classify_registration_error)
    }

    pub async fn purge_unverified_accounts(
        &self,
        expired_before: DateTime<Utc>,
    ) -> Result<Vec<uuid::Uuid>, AccountError> {
        let mut connection = self.connection().await?;
        diesel::delete(
            users::table.filter(
                users::user_id
                    .eq_any(
                        email_verification_tokens::table
                            .select(email_verification_tokens::user_id)
                            .filter(
                                email_verification_tokens::email_verification_token_expires_at
                                    .lt(expired_before),
                            ),
                    )
                    .and(users::user_is_email_verified.eq(false)),
            ),
        )
        .returning(users::user_id)
        .get_results(&mut connection)
        .await
        .map_err(AccountError::Mutation)
    }
}

const EMAIL_UNIQUE_CONSTRAINT: &str = "users_user_email_unique";
const USER_NAME_UNIQUE_CONSTRAINT: &str = "users_user_name_unique";

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
